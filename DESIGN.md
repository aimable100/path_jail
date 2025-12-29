# path_jail Design Document

This document captures design decisions and rationale. For usage, see README.md.

## 1. The Problem

### 1.1 The New File Paradox

The standard approach:

```rust
let path = root.join(user_input).canonicalize()?;
if !path.starts_with(&root) {
    return Err("escape attempt");
}
```

The bug: `canonicalize()` fails if the file doesn't exist. You cannot validate paths for files you intend to create.

### 1.2 The Symlink Trap

An attacker creates:
```
uploads/innocent_link -> /etc
```

Writing to `uploads/innocent_link/passwd` overwrites system files. String-based `..` removal does not catch this.

### 1.3 The Broken Symlink Trap

An attacker creates:
```
uploads/evil -> /etc/shadow  (target doesn't exist yet)
```

`Path::exists()` returns false for broken symlinks. If you skip verification, a later write could follow the symlink to an external location.

### 1.4 The Traversal Trap

Lexical path cleaning is insufficient:
- `foo/../bar` vs `foo/bar`
- Windows: `C:\Users` vs `\\?\C:\Users`

You need OS-level path resolution.

## 2. Security Model

### 2.1 Guarantees

`path_jail` guarantees the returned path was physically inside the jail at the moment of verification.

| Attack | Example | Blocked |
|--------|---------|---------|
| Path traversal | `../../etc/passwd` | Yes |
| Symlink escape | `link -> /etc` | Yes |
| Symlink chains | `a -> b -> /etc` | Yes |
| Broken symlinks | `link -> /nonexistent` | Yes |
| Absolute injection | `/etc/passwd` | Yes |
| Parent escape | `foo/../../secret` | Yes |

### 2.2 Limitations (TOCTOU)

This library validates paths. It does not hold file descriptors.

There is a time-of-check time-of-use race condition. If an attacker has write access to the jail directory, they could swap a directory with a symlink between validation and use.

**Defends against:**
- Logic errors in path construction
- Confused deputy attacks from untrusted input

**Does not defend against:**
- Malicious local processes racing your I/O

For kernel-enforced sandboxing, use `cap-std`.

## 3. API Contract

### 3.1 Core Types

```rust
/// A filesystem sandbox that restricts paths to a root directory.
#[derive(Debug, Clone)]
pub struct Jail {
    root: PathBuf,  // Always canonicalized
}

#[derive(Debug)]
pub enum JailError {
    EscapedRoot { attempted: PathBuf, root: PathBuf },
    BrokenSymlink(PathBuf),
    InvalidPath(String),
    Io(std::io::Error),
}
```

### 3.2 Methods

| Method | Input | Output | Notes |
|--------|-------|--------|-------|
| `Jail::new(root)` | Directory path | `Result<Jail, JailError>` | Root must exist |
| `Jail::root()` | - | `&Path` | Canonicalized root |
| `Jail::join(relative)` | Relative path | `Result<PathBuf, JailError>` | Works for non-existent files |
| `Jail::contains(absolute)` | Absolute path | `Result<PathBuf, JailError>` | Path must exist |
| `Jail::relative(path)` | Absolute or relative | `Result<PathBuf, JailError>` | Strips root prefix |
| `path_jail::join(root, path)` | Root + relative | `Result<PathBuf, JailError>` | One-shot convenience |

### 3.3 Design Decisions

**Why `#[must_use]` on `join()` and `contains()`?**

Prevents confused deputy attacks where the user validates a path but then uses the original untrusted input:

```rust
// WRONG: validates but ignores result
jail.join(user_input)?;
std::fs::write(user_input, data)?;  // Uses unvalidated path!

// RIGHT: uses the validated path
let safe = jail.join(user_input)?;
std::fs::write(&safe, data)?;
```

**Why reject broken symlinks?**

A broken symlink's target cannot be verified. If we returned the path, and the target was later created (or already exists but is inaccessible), the symlink could point outside the jail.

**Why canonicalize the root immediately?**

Ensures `starts_with()` comparisons work correctly. Without canonicalization:
- `/var/uploads` vs `/var/./uploads` would fail
- macOS: `/var` vs `/private/var` would fail

**Why no I/O helpers?**

Keeps the crate focused on path validation. Users can compose with `std::fs`:

```rust
let path = jail.join(input)?;
std::fs::write(&path, data)?;
```

This is more flexible and doesn't hide what's happening.

## 4. Project Structure

```
path_jail/
├── src/
│   ├── lib.rs      # Re-exports, join() convenience function
│   ├── jail.rs     # Jail struct and methods
│   └── error.rs    # JailError enum
├── tests/
│   └── security.rs # Integration tests
├── README.md       # User guide
├── DESIGN.md       # This file
├── LICENSE-MIT
└── LICENSE-APACHE
```

## 5. Future Considerations

Not planned, but possible extensions if there's demand:

- **Async support**: Feature-gated async versions of I/O operations
- **Serde support**: Deserialize `Jail` from config files
- **Custom canonicalization**: For virtual filesystems or testing

These would be feature-gated to maintain zero-dependency default.
