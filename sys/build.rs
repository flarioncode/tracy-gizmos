use std::env;
use std::path::PathBuf;

fn main() {
	let mut tracy = PathBuf::from(
		env::var("CARGO_MANIFEST_DIR").expect("Failed to get the current manifest directory."),
	);
	tracy.push("tracy");

	let defines = defines_from_features();

	#[cfg(feature = "bindgen")]
	{
		let mut out_path = PathBuf::from(
			env::var("CARGO_MANIFEST_DIR").expect("Failed to get the output directory.")
		);
		out_path.push("src");

		let bindings = bindgen::Builder::default()
			.header(tracy.join("tracy/TracyC.h").to_string_lossy())
			.clang_args([
				"-DTRACY_ENABLE",
				"-DTRACY_MANUAL_LIFETIME",
				"-DTRACY_DELAYED_INIT",
				"-DTRACY_NO_FRAME_IMAGE",
				"-DTRACY_NO_VERIFY",
			])
			.clang_args(defines.iter().map(|s| format!("-D{}", s)))
			.allowlist_item("^___tracy.*")
			.allowlist_item("TracyPlot.*")
			.allowlist_item("TracyCZone.*")
			.prepend_enum_name(false)
			.must_use_type("TracyCZoneCtx")
			.explicit_padding(true) // @Speed Re-think if needed.
			.sort_semantically(true)
			.layout_tests(false)
			.merge_extern_blocks(true)
			.generate()
			.expect("Failed to generate bindings.");

		bindings
			.write_to_file(out_path.join("bindings.rs"))
			.expect("Failed to write the bindings.");
	}

	// We can use `pkg_config` to find the library in the system.
	// However, it is not that easy on Windows and dealing with
	// versions might be hairy.

	let mut builder = cc::Build::new();
	builder
		.cpp(true)
		.file(tracy.join("TracyClient.cpp"))
		// We always enable it to simplify things. If profiling is not needed,
		// this crate as a dependency could be optional.
		.define("TRACY_ENABLE",          None)
		.define("TRACY_MANUAL_LIFETIME", None)
		.define("TRACY_DELAYED_INIT",    None)
		.define("TRACY_NO_FRAME_IMAGE",  None)
		.define("TRACY_NO_VERIFY",       None)
		.define("NDEBUG",                None)
		.opt_level(3); // We always optimize as it is important for dev builds, too.

	for define in defines {
		builder.define(define, None);
	}

	builder
		.compile("tracy-client")
}

fn defines_from_features() -> Vec<&'static str> {
	let mut defines = Vec::new();
	if !is_set("CARGO_FEATURE_CRASH_HANDLER") {
		defines.push("TRACY_NO_CRASH_HANDLER");
	}
	if !is_set("CARGO_FEATURE_SYSTEM_TRACING") {
		defines.push("TRACY_NO_SYSTEM_TRACING");
	}
	if !is_set("CARGO_FEATURE_CONTEXT_SWITCH") {
		defines.push("TRACY_NO_CONTEXT_SWITCH");
	}
	if !is_set("CARGO_FEATURE_SAMPLING") {
		defines.push("TRACY_NO_SAMPLING");
	}
	if !is_set("CARGO_FEATURE_CALLSTACK_INLINES") {
		defines.push("TRACY_NO_CALLSTACK_INLINES");
	}
	if !is_set("CARGO_FEATURE_HW_COUNTERS") {
		defines.push("TRACY_NO_SAMPLE_RETIREMENT");
		defines.push("TRACY_NO_SAMPLE_BRANCH");
		defines.push("TRACY_NO_SAMPLE_CACHE");
	}
	if !is_set("CARGO_FEATURE_CODE_TRANSFER") {
		defines.push("TRACY_NO_CODE_TRANSFER");
	}
	if !is_set("CARGO_FEATURE_VSYNC") {
		defines.push("TRACY_NO_VSYNC_CAPTURE");
	}
	if is_set("CARGO_FEATURE_NO_EXIT") {
		defines.push("TRACY_NO_EXIT");
	}
	if !is_set("CARGO_FEATURE_BROADCAST") {
		defines.push("TRACY_NO_BROADCAST");
	}
	if is_set("CARGO_FEATURE_ONLY_LOCALHOST") {
		defines.push("TRACY_ONLY_LOCALHOST");
	}
	if is_set("CARGO_FEATURE_ONLY_IPV4") {
		defines.push("TRACY_ONLY_IPV4");
	}
	defines
}

fn is_set(key: &str) -> bool {
	env::var_os(key).is_some()
}
