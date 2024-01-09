use std::env;
use std::path::PathBuf;

fn main() {
	let mut tracy = PathBuf::from(
		env::var("CARGO_MANIFEST_DIR").expect("Failed to get the current manifest directory."),
	);
	tracy.push("tracy");

	let mut out_path = PathBuf::from(
		env::var("CARGO_MANIFEST_DIR").expect("Failed to get the output directory.")
	);
	out_path.push("src");

	// @Incomplete Feature-gate this, so we can just commit the
	// bindings to the repository and skip requiring the LLVM to build
	// this crate. LLVM is needed due to bindgen's dependency on
	// libclang.
	let bindings = bindgen::Builder::default()
		.header(tracy.join("tracy/TracyC.h").to_string_lossy())
		.clang_args(["-DTRACY_ENABLE", "-DTRACY_MANUAL_LIFETIME"])
		.allowlist_item("^___tracy.*")
		.must_use_type("TracyCZoneCtx")
		.explicit_padding(true) // @Speed Re-think if needed.
		.no_copy("^___tracy.*")
		.sort_semantically(true)
		.layout_tests(false)
		.merge_extern_blocks(true)
		.generate()
		.expect("Failed to generate bindings.");

	bindings
		.write_to_file(out_path.join("bindings.rs"))
		.expect("Failed to write the bindings.");

	// We can use `pkg_config` to find the library in the system.
	// However, it is not that easy on Windows and dealing with
	// versions might be hairy.

	// @Incomplete Expose Tracy features as crate features and setup
	// defines here accordingly. :Features

	// @Incomplete Link dependencies?

	let mut builder = cc::Build::new();
	builder
		.cpp(true)
		.file(tracy.join("TracyClient.cpp"))
		// We always enable it to simplify things. If profiling is not needed,
		// this crate as a dependency could be optional.
		.define("TRACY_ENABLE", None)
		.opt_level(3) // We always optimize as it is important for dev builds, too.
		.compile("tracy-client")
}
