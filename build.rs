fn main() {
    println!("cargo:rustc-link-search=chromium/src/build/linux/debian_bullseye_amd64-sysroot/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-search=chromium/src/build/linux/debian_bullseye_amd64-sysroot/usr/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-search=chromium/src/build/linux/debian_bullseye_amd64-sysroot/usr");
    println!("cargo:rustc-link-search=chromium/src/build/linux/debian_bullseye_amd64-sysroot");
}
