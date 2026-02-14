use std::path::{Path, PathBuf};

fn copy_daemon_binary_if_available() {
    #[cfg(target_os = "windows")]
    let daemon_name = "localdomain-daemon.exe";
    #[cfg(not(target_os = "windows"))]
    let daemon_name = "localdomain-daemon";

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"));
    let workspace_root = manifest_dir
        .parent()
        .expect("src-tauri should have a workspace parent");
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let src = workspace_root
        .join("target")
        .join(&profile)
        .join(daemon_name);
    let dst = workspace_root.join("resources").join(daemon_name);

    // Re-run this build script when the daemon binary changes.
    println!("cargo:rerun-if-changed={}", src.display());

    // If daemon wasn't built yet, keep current behavior but provide guidance.
    if !src.exists() {
        println!(
            "cargo:warning=Daemon binary not found at {}. Build it first: cargo build -p localdomain-daemon{}",
            src.display(),
            if profile == "release" { " --release" } else { "" }
        );
        return;
    }

    let needs_copy = match (std::fs::metadata(&src), std::fs::metadata(&dst)) {
        (Ok(src_meta), Ok(dst_meta)) => {
            src_meta.modified().ok().zip(dst_meta.modified().ok()).map_or(true, |(s, d)| s > d)
        }
        (Ok(_), Err(_)) => true,
        _ => true,
    };

    if needs_copy {
        if let Some(parent) = Path::new(&dst).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::copy(&src, &dst) {
            panic!(
                "Failed to copy daemon binary from {} to {}: {}",
                src.display(),
                dst.display(),
                e
            );
        }
        println!(
            "cargo:warning=Copied daemon binary into bundle resources: {}",
            dst.display()
        );
    }
}

fn main() {
    copy_daemon_binary_if_available();
    tauri_build::build();
}
