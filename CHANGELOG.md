# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-12-29

### Added

- **Security**: Reject null bytes in paths (prevents C string terminator attacks)
- `InvalidRoot` error variant for filesystem root and non-directory detection
- `#[non_exhaustive]` on `JailError` for future compatibility
- Comprehensive edge case tests (38 total)
- Documentation for platform-specific security considerations

### Changed

- **Breaking**: `JailError` is now `#[non_exhaustive]` - add a catch-all arm to matches
- **Breaking**: MSRV bumped from 1.70 to 1.80 (for `LazyLock` in examples)
- `InvalidRoot` provides context-aware error messages ("filesystem root" vs "not a directory")
- Improved documentation with framework examples (Axum, Actix-web)

### Security

- Null byte injection is now blocked (previously passed through for non-existent paths)
- Filesystem roots (`/`, `C:\`) are now rejected at construction

## [0.1.0] - 2024-12-28

### Added

- Initial release
- `Jail` struct for filesystem sandboxing
- `join()` convenience function
- Symlink escape detection
- Broken symlink rejection
- Path traversal prevention

