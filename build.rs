fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        cc::Build::new()
            .file("src/vmx/exit_asm.s")
            .compile("hlidskjalf_exit_asm");
    }

    if let Ok(wdk_path) = std::env::var("WDK_PATH") {
        let search_path = format!("{}/Lib/km/x64", wdk_path);
        println!("cargo:rustc-link-search=native={}", search_path);
        println!("cargo:rustc-link-lib=ntoskrnl");
        println!("cargo:rustc-link-lib=hal");
    } else {
        println!(
            "cargo:warning=WDK_PATH not set. Ensure WDK libraries are available in your system path."
        );
    }
}
