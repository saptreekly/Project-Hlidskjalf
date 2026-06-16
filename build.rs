use std::path::{Path, PathBuf};

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        cc::Build::new()
            .file("src/vmx/exit_asm.asm")
            .compile("hlidskjalf_exit_asm");
    }

    if let Ok(wdk_path) = std::env::var("WDK_PATH") {
        if let Some(search_path) = find_wdk_km_lib_path(&wdk_path) {
            println!(
                "cargo:rustc-link-search=native={}",
                search_path.display()
            );
            println!("cargo:rustc-link-lib=ntoskrnl");
            println!("cargo:rustc-link-lib=hal");
        } else {
            println!(
                "cargo:warning=Could not find WDK km/x64 libs under {wdk_path}. Set WDK_PATH to your Windows Kits root."
            );
        }
    } else {
        println!(
            "cargo:warning=WDK_PATH not set. Ensure WDK libraries are available in your system path."
        );
    }
}

fn find_wdk_km_lib_path(wdk_root: &str) -> Option<PathBuf> {
    let direct = Path::new(wdk_root).join("Lib").join("km").join("x64");
    if direct.is_dir() {
        return Some(direct);
    }

    let lib_root = Path::new(wdk_root).join("Lib");
    let entries = std::fs::read_dir(&lib_root).ok()?;
    let mut candidates = Vec::new();
    for entry in entries.flatten() {
        let km = entry.path().join("km").join("x64");
        if km.is_dir() {
            candidates.push(km);
        }
    }

    candidates.sort();
    candidates.pop()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_km_path_is_preferred() {
        let tmp = std::env::temp_dir().join("hlidskjalf-wdk-test");
        let km = tmp.join("Lib").join("km").join("x64");
        std::fs::create_dir_all(&km).unwrap();
        assert_eq!(find_wdk_km_lib_path(tmp.to_str().unwrap()), Some(km));
        let _ = std::fs::remove_dir_all(tmp);
    }
}
