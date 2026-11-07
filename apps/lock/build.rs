//! On atomic systems the unversioned dev symlink `libpam.so` is
//! missing (no pam-devel). We create it ourselves in OUT_DIR and
//! point the linker there — works on the PC dev build
//! AND in the Fedora builder, without a single extra package.
fn main() {
    let out = std::env::var("OUT_DIR").unwrap();
    for candidate in [
        "/usr/lib64/libpam.so.0",
        "/usr/lib/x86_64-linux-gnu/libpam.so.0",
        "/lib64/libpam.so.0",
    ] {
        if std::path::Path::new(candidate).exists() {
            let link = format!("{out}/libpam.so");
            let _ = std::fs::remove_file(&link);
            std::os::unix::fs::symlink(candidate, &link).ok();
            println!("cargo:rustc-link-search=native={out}");
            break;
        }
    }
    println!("cargo:rustc-link-lib=dylib=pam");
}
