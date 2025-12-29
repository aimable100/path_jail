# path_jail

[![CI](https://github.com/aimable100/path_jail/actions/workflows/ci.yml/badge.svg)](https://github.com/aimable100/path_jail/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/path_jail.svg)](https://crates.io/crates/path_jail)
[![Documentation](https://docs.rs/path_jail/badge.svg)](https://docs.rs/path_jail)
[![License](https://img.shields.io/crates/l/path_jail.svg)](https://github.com/aimable100/path_jail#license)

A zero-dependency filesystem sandbox for Rust. Restricts paths to a root directory, preventing traversal attacks while supporting files that don't exist yet.

## The Problem

The standard approach fails for new files:

```rust
// This breaks if the file doesn't exist yet!
let path = root.join(user_input).canonicalize()?;
if !path.starts_with(&root) {
    return Err("escape attempt");
}
```

## The Solution

```rust
// One-liner for simple cases
let path = path_jail::join("/var/uploads", user_input)?;
std::fs::write(&path, data)?;

// Blocked: returns Err(EscapedRoot)
path_jail::join("/var/uploads", "../../etc/passwd")?;
```

For multiple paths, create a `Jail` and reuse it:

```rust
use path_jail::Jail;

let jail = Jail::new("/var/uploads")?;
let path1 = jail.join("report.pdf")?;
let path2 = jail.join("data.csv")?;
```

## Features

- **Zero dependencies** - only stdlib
- **Symlink-safe** - resolves and validates symlinks
- **Works for new files** - validates paths that don't exist yet
- **Helpful errors** - tells you what went wrong and why

## Security

| Attack | Example | Blocked |
|--------|---------|---------|
| Path traversal | `../../etc/passwd` | Yes |
| Symlink escape | `link -> /etc` | Yes |
| Symlink chains | `a -> b -> /etc` | Yes |
| Broken symlinks | `link -> /nonexistent` | Yes |
| Absolute injection | `/etc/passwd` | Yes |
| Parent escape | `foo/../../secret` | Yes |

### Limitations

This library validates paths. It does not hold file descriptors.

There is a **TOCTOU (time-of-check time-of-use)** race condition. If an attacker has write access to the jail directory, they could swap a directory with a symlink between validation and use.

**Defends against:**
- Logic errors in path construction
- Confused deputy attacks from untrusted input

**Does not defend against:**
- Malicious local processes racing your I/O

For kernel-enforced sandboxing, use [`cap-std`](https://docs.rs/cap-std).

## API

### One-shot validation

```rust
// Validate and join in one call
let safe: PathBuf = path_jail::join("/var/uploads", "subdir/file.txt")?;
```

### Reusable jail

```rust
use path_jail::Jail;

// Create a jail (root must exist and be a directory)
let jail = Jail::new("/var/uploads")?;

// Get the canonicalized root
let root: &Path = jail.root();

// Safely join a relative path
let path: PathBuf = jail.join("subdir/file.txt")?;

// Check if an absolute path is inside the jail
let verified: PathBuf = jail.contains("/var/uploads/file.txt")?;

// Get relative path for database storage
let rel: PathBuf = jail.relative(&path)?;  // "subdir/file.txt"
```

## Want Type-Safe Paths?

If you want to enforce validated paths at compile time, use the newtype pattern:

```rust
use path_jail::{Jail, JailError};
use std::path::{Path, PathBuf};

/// A path verified to be inside a jail.
pub struct JailedPath(PathBuf);

impl JailedPath {
    pub fn new(jail: &Jail, path: impl AsRef<Path>) -> Result<Self, JailError> {
        jail.join(path).map(Self)
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

// Now your functions can require JailedPath
fn save_upload(path: JailedPath, data: &[u8]) -> std::io::Result<()> {
    std::fs::write(path.as_path(), data)
}
```

This makes "confused deputy" bugs a compile error: you cannot accidentally pass an unvalidated `PathBuf` where a `JailedPath` is expected.

## Alternatives

| | path_jail | strict-path | cap-std |
|-|-----------|-------------|---------|
| Approach | Path validation | Type-safe path system | File descriptors |
| Returns | `std::path::PathBuf` | Custom `StrictPath<T>` | Custom `Dir`/`File` |
| Dependencies | 0 | ~5 | ~10 |
| TOCTOU-safe | No | No | Yes |
| Best for | Simple file sandboxing | Complex type-safe paths | Kernel-enforced security |

- [`strict-path`](https://crates.io/crates/strict-path) - More comprehensive, uses marker types for compile-time guarantees
- [`cap-std`](https://docs.rs/cap-std) - Capability-based, TOCTOU-safe, but different API than `std::fs`

## License

MIT OR Apache-2.0

