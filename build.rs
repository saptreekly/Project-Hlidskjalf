// build.rs

fn main() {
    // The WDK_PATH environment variable should point to the root of the WDK installation
    // Example: C:\Program Files (x86)\Windows Kits\10
    if let Ok(wdk_path) = std::env::var("WDK_PATH") {
        // Attempt to find the libraries by searching in the likely subdirectory
        // Since the exact version folder name is causing issues, we can try to
        // build the path dynamically or look for the first matching folder.
        // For now, let's assume standard structure and search recursively or
        // try a slightly more robust path if possible.
        
        // As a fallback to the previous failed path, try a slightly different 
        // structure often found in WDK installations:
        let search_path = format!("{}/Lib/km/x64", wdk_path);
        println!("cargo:rustc-link-search=native={}", search_path);

        // Link against necessary WDK libs for a kernel driver
        println!("cargo:rustc-link-lib=ntoskrnl");
        println!("cargo:rustc-link-lib=hal");
    } else {
        println!(
            "cargo:warning=WDK_PATH not set. Ensure WDK libraries are available in your system path."
        );
    }
}
