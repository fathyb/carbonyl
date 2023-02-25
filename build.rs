#[cfg(target_arch = "x86_64")]
fn link_sysroot() {
    println!("cargo:rustc-link-search=chromium/src/build/linux/debian_bullseye_amd64-sysroot/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-search=chromium/src/build/linux/debian_bullseye_amd64-sysroot/usr/lib/x86_64-linux-gnu");

    println!(
        "cargo:rustc-link-arg=--sysroot=./chromium/src/build/linux/debian_bullseye_amd64-sysroot"
    );
}

#[cfg(target_arch = "x86")]
fn link_sysroot() {
    println!("cargo:rustc-link-search=chromium/src/build/linux/debian_bullseye_i386-sysroot/lib/i386-linux-gnu");
    println!("cargo:rustc-link-search=chromium/src/build/linux/debian_bullseye_i386-sysroot/usr/lib/i386-linux-gnu");

    println!(
        "cargo:rustc-link-arg=--sysroot=./chromium/src/build/linux/debian_bullseye_i386-sysroot"
    );
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
fn link_sysroot() {
    // Intentionally left blank.
}

fn main() {
    link_sysroot();
}
