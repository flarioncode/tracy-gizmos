# 🧰 tracy-gizmos-attributes

[![Latest Version]][crates.io]
[![MIT licensed][mit-badge]][mit-url]
[![Apache licensed][apache-badge]][apache-url]
[![API](https://docs.rs/tracy-gizmos-attributes/badge.svg)][docs.rs]

[Latest Version]: https://img.shields.io/crates/v/tracy-gizmos-attributes.svg
[crates.io]: https://crates.io/crates/tracy-gizmos-attributes
[docs.rs]: https://docs.rs/tracy-gizmos-attributes
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/den-mentiei/tracy-gizmos/blob/main/LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/license-Apache%202.0-blue.svg
[apache-url]: https://github.com/den-mentiei/tracy-gizmos/blob/main/LICENSE-APACHE

## Overview

A procedural macro attribute for instrumenting functions with
[`tracy-gizmos`] zones.

## Usage

In the `Cargo.toml`:

```toml
[dependencies]
tracy-gizmos-attributes = "0.0.1"
```

The `#[instrument]` attribute can now be added to a function to
automatically create and enter a `tracy-gizmos` [zone] when that
function is called. For example:

```rust
#[tracy_gizmos_attributes::instrument]
fn work() {
    // do stuff
}
```

[`tracy-gizmos`]: https://crates.io/crates/tracy-gizmos
[zone]: https://docs.rs/tracy-gizmos/latest/tracy_gizmos/struct.Zone.html

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
