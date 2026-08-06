use std::path::PathBuf;

fn main() {
    pyo3_build_config::add_extension_module_link_args();

    // The lexbor dylib comes from the workspace vcpkg install; add an rpath so
    // the extension module can be imported without DYLD/LD_LIBRARY_PATH.
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.join("../../../..").canonicalize().unwrap();
    let arch = std::env::consts::ARCH.replace("x86_64", "x64").replace("aarch64", "arm64");
    let os = std::env::consts::OS.replace("macos", "osx");
    let lib_dir = repo_root.join("vcpkg_installed").join(format!("{arch}-{os}")).join("lib");
    if lib_dir.is_dir() {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }
}
