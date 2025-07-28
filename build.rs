use std::env;

fn main() {
    // Set build-time environment variables
    println!(
        "cargo:rustc-env=CARGO_PKG_RUST_VERSION={}",
        env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string())
    );

    // Enable static linking on Windows
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:CONSOLE");
    }

    // Optimize for size on release builds
    if env::var("PROFILE").unwrap() == "release" {
        println!("cargo:rustc-link-arg=-s");
    }
}
