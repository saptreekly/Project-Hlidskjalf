// build.rs

fn main() {
    // The WDK_PATH environment variable should point to the root of the WDK installation
    // Example: C:\Program Files (x86)\Windows Kits\10
    if let Ok(wdk_path) = std::env::var("WDK_PATH") {
        // Add WDK library search path
        // Using a more generic path structure to handle different WDK versions
        println!(
            "cargo:rustc-link-search=native={}/Lib/10.0.26100.0/km/x64",
            wdk_path
        );

        // Link against necessary WDK libs for a kernel driver
        println!("cargo:rustc-link-lib=ntoskrnl");
        println!("cargo:rustc-link-lib=hal");
    } else {
        println!(
            "cargo:warning=WDK_PATH not set. Ensure WDK libraries are available in your system path."
        );
    }
}
