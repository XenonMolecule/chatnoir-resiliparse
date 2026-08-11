use std::path::PathBuf;

fn main() {
    pyo3_build_config::add_extension_module_link_args();

    // The lexbor dylib comes from the workspace vcpkg install; add an rpath so
    // the extension module can be imported without DYLD/LD_LIBRARY_PATH.
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.join("../../../..").canonicalize().unwrap();
    let arch = std::env::consts::ARCH.replace("x86_64", "x64").replace("aarch64", "arm64");
    let os = std::env::consts::OS.replace("macos", "osx");
    let triplet = format!("{arch}-{os}");
    let mut rpaths = Vec::new();

    // 1. Workspace-level vcpkg install, when one was made explicitly.
    let lib_dir = repo_root.join("vcpkg_installed").join(&triplet).join("lib");
    if lib_dir.is_dir() {
        rpaths.push(lib_dir);
    }

    // 2. Fallback: the copy `resiliparse-rs/build.rs` installs into its own
    //    OUT_DIR. Without this, a fresh clone links fine but fails at import
    //    with `Library not loaded: @rpath/liblexbor.*`, because nothing ever
    //    creates the workspace-level directory above. OUT_DIR here is
    //    <target>/<profile>/build/<pkg>-<hash>/out, so the sibling build
    //    directories are two levels up.
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let build_dir = PathBuf::from(&out_dir).join("../..").canonicalize().ok();
        if let Some(build_dir) = build_dir {
            if let Ok(entries) = std::fs::read_dir(&build_dir) {
                for entry in entries.flatten() {
                    let cand = entry.path().join("out/vcpkg_installed").join(&triplet).join("lib");
                    if cand.is_dir() {
                        rpaths.push(cand);
                    }
                }
            }
        }
    }

    for path in rpaths {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
    }
}
