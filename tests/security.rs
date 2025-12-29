use path_jail::Jail;
use tempfile::tempdir;
use std::fs;

#[test]
fn blocks_traversal() {
    let dir = tempdir().unwrap();
    let jail = Jail::new(dir.path()).unwrap();

    // Try to escape
    assert!(jail.join("../secret").is_err());
    assert!(jail.join("../../etc/passwd").is_err());
    assert!(jail.join("foo/../../secret").is_err());
}

#[test]
fn allows_safe_new_files() {
    let dir = tempdir().unwrap();
    let jail = Jail::new(dir.path()).unwrap();

    // "new_file.txt" doesn't exist yet, but should be valid
    let path = jail.join("subdir/new_file.txt").unwrap();
    
    // It should be absolute and start with the jail root (which is canonicalized)
    assert!(path.starts_with(jail.root()));
    assert!(path.ends_with("new_file.txt"));
}

#[test]
fn blocks_absolute_input() {
    let dir = tempdir().unwrap();
    let jail = Jail::new(dir.path()).unwrap();
    
    let err = jail.join("/etc/passwd");
    assert!(err.is_err());
}

#[test]
fn allows_internal_parent_navigation() {
    let dir = tempdir().unwrap();
    let jail = Jail::new(dir.path()).unwrap();
    
    // Create a/b directory structure
    fs::create_dir_all(dir.path().join("a/b")).unwrap();
    
    // Navigate with .. but stay inside jail
    let path = jail.join("a/b/../c").unwrap();
    assert!(path.starts_with(jail.root()));
    assert!(path.ends_with("a/c"));
}

#[test]
#[cfg(unix)]
fn catches_symlink_escape() {
    let dir = tempdir().unwrap();
    let jail = Jail::new(dir.path()).unwrap();
    
    // Create a symlink pointing outside the jail
    let link = dir.path().join("evil");
    std::os::unix::fs::symlink("/etc", &link).unwrap();
    
    // Attempting to traverse through the symlink should fail
    assert!(jail.join("evil/passwd").is_err());
}

#[test]
#[cfg(unix)]
fn allows_internal_symlinks() {
    let dir = tempdir().unwrap();
    let jail = Jail::new(dir.path()).unwrap();
    
    // Create a real directory and a symlink pointing to it (inside jail)
    fs::create_dir(dir.path().join("real")).unwrap();
    std::os::unix::fs::symlink(
        dir.path().join("real"),
        dir.path().join("link")
    ).unwrap();
    
    // Symlink inside jail should be allowed
    let path = jail.join("link").unwrap();
    assert!(path.starts_with(jail.root()));
}

#[test]
fn blocks_dot_dot_at_root() {
    let dir = tempdir().unwrap();
    let jail = Jail::new(dir.path()).unwrap();
    
    // Even a single .. at root should fail
    assert!(jail.join("..").is_err());
}

#[test]
fn handles_dot_components() {
    let dir = tempdir().unwrap();
    let jail = Jail::new(dir.path()).unwrap();
    
    // Current dir components should be ignored
    let path = jail.join("./foo/./bar").unwrap();
    assert!(path.ends_with("foo/bar"));
}