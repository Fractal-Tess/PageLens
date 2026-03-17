use glob::glob;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest_dir.clone());
    let app_dir = repo_root.join("apps/app");

    let status = Command::new("bun")
        .arg("run")
        .arg("--cwd")
        .arg(&app_dir)
        .arg("build")
        .status();

    match status {
        Ok(status) if status.success() => {}
        Ok(status) => panic!("failed to build apps/app static bundle, exit code: {status}"),
        Err(err) => panic!("failed to execute bun for apps/app build: {err}"),
    }

    println!("cargo:rerun-if-changed={}", app_dir.join("src").display());
    println!(
        "cargo:rerun-if-changed={}",
        app_dir.join("static").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        app_dir.join("svelte.config.js").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        app_dir.join("package.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repo_root.join("bun.lock").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        app_dir.join("vite.config.ts").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        app_dir.join("tsconfig.json").display()
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_default());
    let embedded_dir = out_dir.join("pagelens-web");
    if embedded_dir.exists() {
        fs::remove_dir_all(&embedded_dir).expect("failed to clear embedded web output directory");
    }
    fs::create_dir_all(&embedded_dir).expect("failed to create embedded web output directory");

    let built_dir = app_dir.join("build");
    if !built_dir.exists() {
        panic!("apps/app/build not found after build");
    }

    copy_recursive(&built_dir, &embedded_dir).expect("failed to copy web build into OUT_DIR");
}

fn copy_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    let pattern = format!("{}/**/*", src.display());
    for entry in glob(&pattern).expect("invalid glob pattern").flatten() {
        let relative = entry.strip_prefix(src).expect("strip prefix failed");
        let target = dst.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&entry, &target)?;
        }
    }
    Ok(())
}
