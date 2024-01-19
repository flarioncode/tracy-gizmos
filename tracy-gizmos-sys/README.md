# 🧰 tracy-gizmos-sys

[![Latest Version]][crates.io]
[![MIT licensed][mit-badge]][mit-url]
[![Apache licensed][apache-badge]][apache-url]
[![API](https://docs.rs/tracy-gizmos-sys/badge.svg)][docs.rs]

[Latest Version]: https://img.shields.io/crates/v/tracy-gizmos-sys.svg
[crates.io]: https://crates.io/crates/tracy-gizmos-sys
[docs.rs]: https://docs.rs/tracy-gizmos-sys
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/den-mentiei/tracy-gizmos/blob/main/LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/license-Apache%202.0-blue.svg
[apache-url]: https://github.com/den-mentiei/tracy-gizmos/blob/main/LICENSE-APACHE

## Overview

We maintain a carbon copy of Tracy's public part, so we can build it
as a static library and also bindgen the low-level bindings based on
its public header.

Instead of submodules or manual copying, we are using `git-subtree` to
do this. It worth revisiting this as local checkout and 1 copy command
might be easier.

## Bindings

Bindings are generated via `bindgen` and are commited as a `src/`
part. It allows to build this crate quickly and without LLVM (bindgen
requires libclang to do its thing).

When `tracy/` is updated, the bindings should be regenerated:
``` sh
$ cargo clean && cargo build -F bindgen
```

## Initial setup

Just do the following from the git repository root:

```sh
# adding tracy remote and checking out its master in a staging branch
$ git remote add -f tracy-upstream git@github.com:wolfpld/tracy.git
$ git checkout -b staging-tracy tracy-upstream/master

# split off a subdirectory 'public' from its master into a separate branch
$ git subtree split --squash -P public --annotate="Tracy: " --rejoin -b tracy-public

# checkout our main and add 'public' parts above to our 'sys/tracy'
$ git checkout main
$ git subtree add -P sys/tracy --squash tracy-public
```

## How to update

Just do the following from the git repository root:

```sh
# switch back to the tracy's master and update it
$ git checkout staging-tracy
$ git pull tracy-upstream master

# update the subdirectory branch with changes received above
$ git subtree split -P public --annotate="Tracy: " --rejoin -b tracy-public

# checkout our main and merge new 'public' parts to update our 'sys/tracy'
$ git subtree merge -P sys/tracy --squash tracy-public
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Note that the Tracy public part, that this crate uses and embeds, is
licensed under the [3-clause BSD license](LICENSE-tracy).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
