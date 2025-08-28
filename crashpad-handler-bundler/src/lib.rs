use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Primary API - bundles handler to the default target directory.
///
/// - If `CRASHPAD_HANDLER` env is set, copy from that absolute path.
/// - Else, if the destination already exists, ensure permissions on Unix and return it.
/// - Else, emit an error guiding the user to enable crashpad build or set the env var.
/// - Prints minimal cargo metadata for rebuilds and optional runtime default path.
pub fn bundle() -> io::Result<PathBuf> {
    println!("cargo:rerun-if-env-changed=CRASHPAD_HANDLER");
    println!("cargo:rerun-if-env-changed=DEP_CRASHPAD_HANDLER");
    println!("cargo:rerun-if-env-changed=DEP_CRASHPAD_RS_HANDLER");
    // Destination is always computed from the consumer's environment.

    let dest = default_dest_path()?;
    if let Ok(src) = env::var("CRASHPAD_HANDLER") {
        let src_path = PathBuf::from(src);
        validate_source(&src_path)?;
        copy_atomic(&src_path, &dest)?;
        set_exec_permissions_unix(&dest)?;
        println!("cargo:rustc-env=CRASHPAD_HANDLER_PATH={}", dest.display());
        println!("cargo:rerun-if-changed={}", src_path.display());
        println!(
            "cargo:warning=crashpad_handler copied to {}",
            dest.display()
        );
        return Ok(dest);
    }

    if let Ok(src) = env::var("DEP_CRASHPAD_HANDLER") {
        let src_path = PathBuf::from(src);
        validate_source(&src_path)?;
        copy_atomic(&src_path, &dest)?;
        set_exec_permissions_unix(&dest)?;
        println!("cargo:rustc-env=CRASHPAD_HANDLER_PATH={}", dest.display());
        println!("cargo:rerun-if-changed={}", src_path.display());
        println!(
            "cargo:warning=crashpad_handler copied to {}",
            dest.display()
        );
        return Ok(dest);
    }

    if dest.exists() {
        set_exec_permissions_unix(&dest)?;
        println!("cargo:rustc-env=CRASHPAD_HANDLER_PATH={}", dest.display());
        // Handler already exists, no warning needed
        return Ok(dest);
    }

    if let Ok(src) = env::var("DEP_CRASHPAD_RS_HANDLER") {
        // handle pass-through from crashpad crate
        let src_path = PathBuf::from(src);
        validate_source(&src_path)?;
        copy_atomic(&src_path, &dest)?;
        set_exec_permissions_unix(&dest)?;
        println!("cargo:rustc-env=CRASHPAD_HANDLER_PATH={}", dest.display());
        println!("cargo:rerun-if-changed={}", src_path.display());
        println!(
            "cargo:warning=crashpad_handler copied to {}",
            dest.display()
        );
        return Ok(dest);
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "crashpad_handler not found. Set CRASHPAD_HANDLER or depend on a crate exposing DEP_CRASHPAD_HANDLER (e.g., crashpad-rs-sys via crashpad). Expected at {}",
            dest.display()
        ),
    ))
}

/// Find handler without bundling. Returns destination if present or the env-provided source.
pub fn find() -> io::Result<PathBuf> {
    if let Ok(src) = env::var("CRASHPAD_HANDLER") {
        let p = PathBuf::from(src);
        validate_source(&p)?;
        return Ok(p);
    }
    if let Ok(src) = env::var("DEP_CRASHPAD_HANDLER") {
        let p = PathBuf::from(src);
        validate_source(&p)?;
        return Ok(p);
    }
    if let Ok(src) = env::var("DEP_CRASHPAD_RS_HANDLER") {
        let p = PathBuf::from(src);
        validate_source(&p)?;
        return Ok(p);
    }
    let dest = default_dest_path()?;
    if dest.exists() {
        return Ok(dest);
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "crashpad_handler not found at {} and CRASHPAD_HANDLER/DEP_CRASHPAD_HANDLER not set",
            dest.display()
        ),
    ))
}

/// Bundle to a custom directory. Uses OS-default filename. Returns final file path.
pub fn bundle_to(dest_dir: &Path) -> io::Result<PathBuf> {
    fs::create_dir_all(dest_dir)?;
    let name = handler_basename_for_target();
    let dest = dest_dir.join(name);
    println!("cargo:rerun-if-env-changed=CRASHPAD_HANDLER");
    println!("cargo:rerun-if-env-changed=DEP_CRASHPAD_HANDLER");
    println!("cargo:rerun-if-env-changed=DEP_CRASHPAD_RS_HANDLER");

    if let Ok(src) = env::var("CRASHPAD_HANDLER") {
        let src_path = PathBuf::from(src);
        validate_source(&src_path)?;
        copy_atomic(&src_path, &dest)?;
        set_exec_permissions_unix(&dest)?;
        println!("cargo:rustc-env=CRASHPAD_HANDLER_PATH={}", dest.display());
        println!("cargo:rerun-if-changed={}", src_path.display());
        println!(
            "cargo:warning=crashpad_handler copied to {}",
            dest.display()
        );
        return Ok(dest);
    }

    if let Ok(src) = env::var("DEP_CRASHPAD_HANDLER") {
        let src_path = PathBuf::from(src);
        validate_source(&src_path)?;
        copy_atomic(&src_path, &dest)?;
        set_exec_permissions_unix(&dest)?;
        println!("cargo:rustc-env=CRASHPAD_HANDLER_PATH={}", dest.display());
        println!("cargo:rerun-if-changed={}", src_path.display());
        println!(
            "cargo:warning=crashpad_handler copied to {}",
            dest.display()
        );
        return Ok(dest);
    }

    if dest.exists() {
        set_exec_permissions_unix(&dest)?;
        println!("cargo:rustc-env=CRASHPAD_HANDLER_PATH={}", dest.display());
        // Handler already exists, no warning needed
        return Ok(dest);
    }

    if let Ok(src) = env::var("DEP_CRASHPAD_RS_HANDLER") {
        let src_path = PathBuf::from(src);
        validate_source(&src_path)?;
        copy_atomic(&src_path, &dest)?;
        set_exec_permissions_unix(&dest)?;
        println!("cargo:rustc-env=CRASHPAD_HANDLER_PATH={}", dest.display());
        println!("cargo:rerun-if-changed={}", src_path.display());
        println!(
            "cargo:warning=crashpad_handler copied to {}",
            dest.display()
        );
        return Ok(dest);
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "crashpad_handler not found. Provide CRASHPAD_HANDLER or depend on provider of DEP_CRASHPAD_HANDLER to bundle into {}",
            dest.display()
        ),
    ))
}

// --- helpers ---

fn default_dest_path() -> io::Result<PathBuf> {
    let root = target_root_dir()?;
    let triple_dir = if is_cross_compile() {
        Some(env::var("TARGET").unwrap_or_default())
    } else {
        None
    };
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let dir = match triple_dir {
        Some(t) if !t.is_empty() => root.join(t).join(profile),
        _ => root.join(profile),
    };
    fs::create_dir_all(&dir)?;
    Ok(dir.join(handler_basename_for_target()))
}

// No preferred dest: provider's dest is irrelevant for consumer; always compute locally.

fn handler_basename_for_target() -> &'static str {
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("android") {
        "libcrashpad_handler.so"
    } else if target.contains("windows") {
        "crashpad_handler.exe"
    } else {
        "crashpad_handler"
    }
}

fn is_cross_compile() -> bool {
    let host = env::var("HOST").unwrap_or_default();
    let target = env::var("TARGET").unwrap_or_default();
    !host.is_empty() && !target.is_empty() && host != target
}

fn target_root_dir() -> io::Result<PathBuf> {
    if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        return Ok(PathBuf::from(dir));
    }
    if let Ok(out) = env::var("OUT_DIR") {
        // Typical OUT_DIR: .../target/<triple?>/<profile>/build/<pkg>/out
        let mut p = PathBuf::from(out);
        for _ in 0..5 {
            if p.file_name().map(|s| s == "target").unwrap_or(false) {
                return Ok(p);
            }
            if !p.pop() {
                break;
            }
        }
    }
    // Fallback: manifest_dir/target
    if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        let mut p = PathBuf::from(manifest);
        // Most projects keep target at workspace root, parent of package
        if p.pop() {
            return Ok(p.join("target"));
        }
        return Ok(PathBuf::from("target"));
    }
    Ok(PathBuf::from("target"))
}

fn copy_atomic(src: &Path, dest: &Path) -> io::Result<()> {
    // If identical size and mtime, skip
    if let (Ok(sm), Ok(dm)) = (fs::metadata(src), fs::metadata(dest)) {
        let same_size = sm.len() == dm.len();
        let same_mtime = sm
            .modified()
            .ok()
            .zip(dm.modified().ok())
            .map(|(a, b)| a == b)
            .unwrap_or(false);
        if same_size && same_mtime {
            return Ok(());
        }
    }
    let parent = dest
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no parent dir"))?;
    fs::create_dir_all(parent)?;
    let mut tmp = dest.to_path_buf();
    tmp.set_extension("tmp");
    fs::copy(src, &tmp)?;
    // Best-effort rename
    match fs::rename(&tmp, dest) {
        Ok(_) => Ok(()),
        Err(_) => {
            // Fallback: remove and copy
            let _ = fs::remove_file(dest);
            fs::copy(src, dest).map(|_| ())
        }
    }
}

#[cfg(unix)]
fn set_exec_permissions_unix(p: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(p) {
        let mut perm = meta.permissions();
        let mode = perm.mode();
        if mode & 0o111 == 0 {
            perm.set_mode(mode | 0o111);
            fs::set_permissions(p, perm)?;
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_exec_permissions_unix(_p: &Path) -> io::Result<()> {
    Ok(())
}

fn validate_source(p: &Path) -> io::Result<()> {
    if !p.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("source not found: {}", p.display()),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    fn write_dummy_handler(dir: &Path, name: &str) -> PathBuf {
        let p = dir.join(name);
        fs::write(&p, b"dummy").unwrap();
        p
    }

    fn clear_env(keys: &[&str]) {
        for k in keys {
            std::env::remove_var(k);
        }
    }

    #[test]
    fn bundle_with_explicit_env() {
        let _g = ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();
        let td_src = TempDir::new().unwrap();
        let td_dst = TempDir::new().unwrap();
        let name = handler_basename_for_target();
        let src = write_dummy_handler(td_src.path(), name);

        clear_env(&[
            "CRASHPAD_HANDLER",
            "DEP_CRASHPAD_HANDLER",
            "DEP_CRASHPAD_RS_HANDLER",
        ]);
        std::env::set_var("CRASHPAD_HANDLER", &src);

        let out = bundle_to(td_dst.path()).expect("bundle ok");
        assert!(out.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&out).unwrap().permissions().mode();
            assert!(mode & 0o111 != 0, "exec bit should be set");
        }
    }

    #[test]
    fn bundle_with_dep_crashpad_handler() {
        let _g = ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();
        let td_src = TempDir::new().unwrap();
        let td_dst = TempDir::new().unwrap();
        let name = handler_basename_for_target();
        let src = write_dummy_handler(td_src.path(), name);

        clear_env(&[
            "CRASHPAD_HANDLER",
            "DEP_CRASHPAD_HANDLER",
            "DEP_CRASHPAD_RS_HANDLER",
        ]);
        std::env::set_var("DEP_CRASHPAD_HANDLER", &src);

        let out = bundle_to(td_dst.path()).expect("bundle ok");
        assert!(out.exists());
    }

    #[test]
    fn bundle_with_dep_crashpad_rs_handler() {
        let _g = ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();
        let td_src = TempDir::new().unwrap();
        let td_dst = TempDir::new().unwrap();
        let name = handler_basename_for_target();
        let src = write_dummy_handler(td_src.path(), name);

        clear_env(&[
            "CRASHPAD_HANDLER",
            "DEP_CRASHPAD_HANDLER",
            "DEP_CRASHPAD_RS_HANDLER",
        ]);
        std::env::set_var("DEP_CRASHPAD_RS_HANDLER", &src);

        let out = bundle_to(td_dst.path()).expect("bundle ok");
        assert!(out.exists());
    }

    #[test]
    fn find_prefers_envs() {
        let _g = ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();
        let td_src = TempDir::new().unwrap();
        let name = handler_basename_for_target();
        let src = write_dummy_handler(td_src.path(), name);
        clear_env(&[
            "CRASHPAD_HANDLER",
            "DEP_CRASHPAD_HANDLER",
            "DEP_CRASHPAD_RS_HANDLER",
        ]);
        std::env::set_var("CRASHPAD_HANDLER", &src);
        let p = find().expect("find ok");
        assert_eq!(p, src);
    }

    #[test]
    fn error_when_no_source_and_no_dest() {
        let _g = ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();
        clear_env(&[
            "CRASHPAD_HANDLER",
            "DEP_CRASHPAD_HANDLER",
            "DEP_CRASHPAD_RS_HANDLER",
        ]);
        let td_dst = TempDir::new().unwrap();
        // Ensure empty directory
        let res = bundle_to(td_dst.path());
        assert!(res.is_err());
    }
}
