# TODO

- [ ] enable function names only on nightly via `build.rs` and `cargo:rustc-cfg=nightly`
- [ ] frame! at the end vs Frame+Drop (it is optional!)
- [ ] discontinuous frames aka frame start/end pair with same name pointer
- [ ] document everything
- [ ] what's up with locks & C API
- [ ] what's up with alloc & free, named overloads
- [ ] tracy app info (text, size)
- [ ] callstacks! depth is at most 62 could be disabled with TRACY_NO_CALLSTACK, TRACY_NO_CALLSTACK_INLINES
- [ ] auto-function proc-macro attributes:
	- [ ] #[zone]
	- [ ] #[zone(name)]
	- [ ] #[zone(color)]
	- [ ] #[zone(name, color)]
	- [ ] + callstacks?! + enabled
- [ ] gfx things
- [ ] dbghelp thread-safety on windows
- [x] crate examples
- [x] formatting only works with one argument
- [x] TracyMessageL gets static
      TracyMessage(text, size) no terminating zero and can't be larger than 64 Kb. will be copied.
- [x] tests
- [x] :Features
	- [x] `TRACY_NO_CRASH_HANDLER`  exposed as a feature
	- [x] `TRACY_NO_SYSTEM_TRACING` same
	- [x] `TRACY_NO_CONTEXT_SWITCH` same
	- [x] `TRACY_NO_SAMPLING`       same
	- [x] `TRACY_NO_CODE_TRANSFER`  same
	- [x] `TRACY_NO_VSYNC_CAPTURE`  same
- [x] actually use colors
- [x] plots
- [x] basic things
- [x] basic features
- [x] define `TRACY_NO_FRAME_IMAGE` as it is not interesing for now
- [x] define `TRACY_NO_VERIFY` (verify should be exposed as a feature)
- [x] expose colours from `common/TracyColors.hpp` (bindgen try
      failed, too complex to setup, just copypasta it)
