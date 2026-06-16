#![no_main]
use hlidskjalf::check_vmx_support_from_ecx;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let ecx = if data.len() >= 4 {
        u32::from_le_bytes([data[0], data[1], data[2], data[3]])
    } else {
        0
    };
    let _ = check_vmx_support_from_ecx(ecx);
});
