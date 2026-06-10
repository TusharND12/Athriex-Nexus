fn main() {
    // libgit2 on Windows requires Advapi32 for security/token APIs used in tests and binaries.
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=advapi32");
}
