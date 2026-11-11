//! Secure file handling for secret-bearing local files.
//!
//! Everything here errs on the side of refusing to touch a file rather than
//! doing the user a favor they did not ask for. A file is only ever read or
//! written when it is a regular file, not a symlink, owned by the current
//! user, and free of group/other permission bits. Writes go through a temp
//! file + `rename` (never through a symlink), and are always created `0600`
//! (directories `0700`) regardless of the process umask.

use std::{
    fs::{self},
    io::{self, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

pub const SECRET_FILE_MODE: u32 = 0o600;
pub const SECRET_DIR_MODE: u32 = 0o700;

fn euid() -> u32 {
    // SAFETY: geteuid is always safe; it has no failure mode.
    unsafe { libc::geteuid() }
}

/// Check the metadata of `path` and enforce the secret-file rules.
fn check_metadata(path: &Path, meta: &fs::Metadata) -> io::Result<()> {
    let path_str = path.display().to_string();

    if meta.file_type().is_symlink() {
        return Err(unsafe_file(&path_str, "is a symlink"));
    }

    if !meta.is_file() {
        return Err(unsafe_file(&path_str, "is not a regular file"));
    }

    if meta.uid() != euid() {
        return Err(unsafe_file(&path_str, "is not owned by you"));
    }

    let mode = meta.mode() & 0o777;

    if mode & 0o077 != 0 {
        return Err(unsafe_file(
            &path_str,
            &format!("has permissions {:03o}; chmod 600 '{}' before continuing", mode, path_str),
        ));
    }

    Ok(())
}

fn unsafe_file(path_str: &str, reason: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!("refusing to use '{path_str}' because it {reason}"),
    )
}

/// Verify `path` is safe to read/write as a secret file, or explain why not.
pub fn check_secret_file(path: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;

    check_metadata(path, &meta)
}

/// Read a secret file; the file must exist and pass the secret-file checks.
pub fn safe_read(path: &Path) -> io::Result<String> {
    match safe_read_opt(path, true)? {
        Some(content) => Ok(content),
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("'{}' does not exist", path.display()),
        )),
    }
}

/// Read a secret file, optionally returning `None` when it does not exist.
pub fn safe_read_opt(path: &Path, allow_missing: bool) -> io::Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            check_metadata(path, &meta)?;
            Ok(Some(fs::read_to_string(path)?))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound && allow_missing => Ok(None),
        Err(error) => Err(error),
    }
}

/// Create (or tighten) a directory so only the owner can enter/list it.
pub fn ensure_secure_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;

    let meta = fs::metadata(dir)?;

    if !meta.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("'{}' exists but is not a directory", dir.display()),
        ));
    }

    fs::set_permissions(dir, fs::Permissions::from_mode(SECRET_DIR_MODE))
}

/// Write `content` to `path` atomically, always as a `0600` regular file.
///
/// Any existing target is checked first; a symlink at `path` is replaced
/// wholesale, never written through.
pub fn atomic_write_secure(path: &Path, content: &str) -> io::Result<()> {
    if path.exists() {
        check_secret_file(path)?;
    }

    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;

    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(content.as_bytes())?;
    temp.as_file().sync_all()?;

    fs::set_permissions(temp.path(), fs::Permissions::from_mode(SECRET_FILE_MODE))?;
    temp.persist(path).map_err(|error| error.error)?;

    Ok(())
}

/// Validate that `path` cannot escape the current working directory.
///
/// The parent directory is canonicalized against the cwd; paths escaping the
/// project root are refused.
pub fn path_inside_cwd(path: &Path) -> io::Result<()> {
    let cwd = fs::canonicalize(std::env::current_dir()?)?;

    // Canonicalize the nearest existing ancestor of `path` (the parent may
    // not exist yet, e.g. a not-yet-created `.sharenv/` directory).
    let mut anchor = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .to_path_buf();

    let parent = loop {
        match fs::canonicalize(&anchor) {
            Ok(path) => break path,
            Err(error) => {
                if error.kind() != io::ErrorKind::NotFound {
                    return Err(io::Error::new(
                        error.kind(),
                        format!("cannot canonicalize '{}': {error}", anchor.display()),
                    ));
                }

                anchor = match anchor.parent() {
                    Some(parent) if parent != anchor => {
                        if parent.as_os_str().is_empty() {
                            PathBuf::from(".")
                        } else {
                            parent.to_path_buf()
                        }
                    }
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("cannot canonicalize '{}': not found", path.display()),
                        ));
                    }
                };
            }
        }
    };

    if !parent.starts_with(&cwd) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("refusing to touch '{}': it escapes the working directory", path.display()),
        ));
    }

    Ok(())
}
