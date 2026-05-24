#![no_main]
use hlidskjalf::check_vmx_support;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz the CPUID-based VMX support check
    // We can use the input `data` to simulate different CPUID return values
    // if we refactor `check_vmx_support` to accept inputs.
    // For now, we just exercise the code path.
    let _ = check_vmx_support();
});
