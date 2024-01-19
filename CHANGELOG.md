# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2024-xx-xx

### Added
### Fixed
### Changed
### Removed

## [0.0.8] - 2024-01-20

### Added

- `#[capture]` attribute to instrument main added.

### Changed

- minor example updates

## [0.0.7] - 2024-01-19

### Fixed

- `docs.rs` build & intra-crate version dependencies fixed.

## [0.0.6] - 2024-01-19

### Added

- `#[instrument]` attribute to instrument functions automatically.
- attributes example.

### Changed

- `TracyClient::new` is gone, use `start_capture()` instead.
- `TracyClient` renamed to `TracyCapture`.

### Fixed

- Changelog comparison-between-releases links are correct now.

## [0.0.5] - 2024-01-17

### Fixed

- `message!` behaviour with `enabled` is now correct, too.

## [0.0.4] - 2024-01-17

### Changed

- `make_plot!` now accepts the plot variable name to define.
- `plot!` now accepts the plot variable name to emit value to.

### Fixed

- macroses were not properly handling `enabled` at all.
- docs typos and absence in some places due to feature gating.
- using code now nicely compiles without warnings for both `enabled`
  and not cases.

## [0.0.3] - 2024-01-17

### Fixed

- use of the `zone!` without `use` now works properly.

## [0.0.2] - 2024-01-17

### Fixed

- Readmes, comments.
- `is_connected` is no longer `const` when instrumentation is disabled.
- Missing docs are now back.

## [0.0.1] - 2024-01-16

### Added

- Initial implementation of zones, frames, plots & messages

[unreleased]: https://github.com/den-mentiei/tracy-gizmos/compare/v0.0.7...HEAD
[0.0.7]: https://github.com/den-mentiei/tracy-gizmos/releases/tag/v0.0.6..v0.0.7
[0.0.6]: https://github.com/den-mentiei/tracy-gizmos/releases/tag/v0.0.5..v0.0.6
[0.0.5]: https://github.com/den-mentiei/tracy-gizmos/releases/tag/v0.0.4..v0.0.5
[0.0.4]: https://github.com/den-mentiei/tracy-gizmos/compare/v0.0.3..v0.0.4
[0.0.3]: https://github.com/den-mentiei/tracy-gizmos/compare/v0.0.2..v0.0.3
[0.0.2]: https://github.com/den-mentiei/tracy-gizmos/compare/v0.0.1..v0.0.2
[0.0.1]: https://github.com/den-mentiei/tracy-gizmos/releases/tag/v0.0.1
