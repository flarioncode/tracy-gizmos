# 🧰 tracy-gizmos

[![Latest Version]][crates.io]
[![MIT licensed][mit-badge]][mit-url]
[![Apache licensed][apache-badge]][apache-url]
[![API](https://docs.rs/tracy-gizmos/badge.svg)][docs.rs]

[Latest Version]: https://img.shields.io/crates/v/tracy-gizmos.svg
[crates.io]: https://crates.io/crates/tracy-gizmos
[docs.rs]: https://docs.rs/tracy-gizmos
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/den-mentiei/tracy-gizmos/blob/main/LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/license-Apache%202.0-blue.svg
[apache-url]: https://github.com/den-mentiei/tracy-gizmos/blob/main/LICENSE-APACHE

## Overview

Bindings for the client library of the
[Tracy](https://github.com/wolfpld/tracy) profiler.

Allows to easily instrument your code and profile it with Tracy.

## How to use

Add `tracy-gizmos` to your `Cargo.toml`:

```toml
[dependencies]
tracy-gizmos = { version = "0.0.1", features = ["enabled"] }
```

The usage is pretty straight-forward (for more details refer to the docs):

```rust
use tracy_gizmos::*;
fn main() {
	let tracy = TracyClient::start();
	zone!("main");
	work();
}
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Note that the Tracy public part, that this crate uses and embeds, is
licensed under the [3-clause BSD license](sys/LICENSE-tracy).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
