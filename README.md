# 🧰 tracy gizmos

`@Incomplete`

## Overview

`@Incomplete`

## How to use

`@Incomplete`

## TODO

- [x] actually use colors
- [x] plots
- [ ] messages
- [ ] examples
- [ ] tests
- [ ] `TRACY_NO_CRASH_HANDLER`  exposed as a feature
- [ ] `TRACY_NO_SYSTEM_TRACING` same
- [ ] `TRACY_NO_CONTEXT_SWITCH` same
- [ ] `TRACY_NO_SAMPLING`       same
- [ ] `TRACY_NO_CODE_TRANSFER`  same
- [ ] `TRACY_NO_VSYNC_CAPTURE`  same
- [ ] :Features
- [ ] gfx things
- [ ] auto-function proc-macro attributes:
	- [ ] #[zone]
	- [ ] #[zone(name)]
	- [ ] #[zone(color)]
	- [ ] #[zone(name, color)]
	- [ ] + callstacks?! + enabled
	- [ ] fn foo() {}
- [x] basic things
- [x] basic features
- [x] define `TRACY_NO_FRAME_IMAGE` as it is not interesing for now
- [x] define `TRACY_NO_VERIFY` (verify should be exposed as a feature)
- [x] expose colours from `common/TracyColors.hpp` (bindgen try
      failed, too complex to setup, just copypasta it)

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Note that the Tracy public part, that this crate uses and embeds, is
licensed under the [3-clause BSD license](sys/LICENSE-tracy).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
