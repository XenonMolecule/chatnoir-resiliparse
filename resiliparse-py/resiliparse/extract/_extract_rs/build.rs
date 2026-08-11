use std::path::PathBuf;
use std::process::Command;

fn main() {
    pyo3_build_config::add_extension_module_link_args();

    // The extension module loads lexbor at import time, so the built dylib
    // needs an rpath pointing at a lexbor that still exists on disk.
    //
    // Build-script order is not guaranteed to give us one: the copy that
    // `resiliparse-rs/build.rs` installs lives in *its* OUT_DIR, which may not
    // exist yet when this script runs. Relying on it produced a library that
    // linked fine but failed at import from a clean clone with
    // `Library not loaded: @rpath/liblexbor.*`.
    //
    // Instead, materialize a workspace-level vcpkg tree ourselves and point the
    // rpath there. vcpkg is a no-op (cache hit) when the packages are already
    // installed, so this costs nothing on rebuilds.
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.join("../../../..").canonicalize().unwrap();
    let arch = std::env::consts::ARCH.replace("x86_64", "x64").replace("aarch64", "arm64");
    let os = std::env::consts::OS.replace("macos", "osx");
    let triplet = std::env::var("VCPKG_DEFAULT_TRIPLET").unwrap_or_else(|_| format!("{arch}-{os}"));

    let install_root = repo_root.join("vcpkg_installed");
    let lib_dir = install_root.join(&triplet).join("lib");

    if !lib_dir.is_dir() {
        let out = Command::new("vcpkg")
            .args([
                "install",
                "--triplet",
                &triplet,
                "--x-install-root",
                &install_root.display().to_string(),
            ])
            .current_dir(&repo_root)
            .env("CMAKE_POLICY_VERSION_MINIMUM", "3.5")
            .output();
        match out {
            Ok(o) if !o.status.success() => panic!(
                "vcpkg install failed while preparing lexbor for the extension module:\n{}\n{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => panic!("failed to run vcpkg (is it on PATH?): {e}"),
            _ => {}
        }
    }

    if lib_dir.is_dir() {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
}
