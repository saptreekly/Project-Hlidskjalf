# Project Hliðskjálf

"Hliðskjálf... the high throne of the Allfather, from which all realms are watched."

Project Hliðskjálf is an open-source, ultra-lightweight Type-1.5 thin hypervisor. It is bare-metal systems architecture engineered to provide immutable, hardware-level security retrofits for legacy End-of-Life (EOL) x86_64 operating systems, starting with Windows 10.

## The Philosophy

This project is a direct tactical response to the macroeconomic vulnerability and forced compliance engineered by Microsoft's sunsetting of Windows 10. Mandating a migration to Windows 11 (a bloated, telemetry-heavy ecosystem that arbitrarily discards fully functional hardware) violates the core tenets of technical sovereignty and sustainability.

In my Norse faith, we honor self-reliance, the preservation of our tools, and unwavering vigilance of our own boundaries. I do not accept the forced obsolescence of our craft at the whim of a centralized corporate directive.

As Odin observes the Nine Realms from his high seat, Hliðskjálf establishes an unyielding sentinel layer. We do not beg a compromised or unsupported operating system kernel for security permissions. Instead, we subvert the architecture on-the-fly, dropping the host kernel into a hardware-enforced virtual machine sandbox at Ring-1. From this high throne, we audit, protect, and police the entire operating system from above.

## Architecture

- **Zero Third-Party Dependencies:** The entire execution core is monitored completely within bare-metal Rust with zero reliance on standard libraries, allocation abstractions, or OS runtimes.
- **On-the-Fly Subversion:** The driver dynamically virtualizes a live, running host kernel without requiring system reboots or pre-boot adjustments, natively initializing Intel VT-x extensions via tight inline x86_64 Assembly.
- **EPT Memory Shadowing:** Leverages Extended Page Tables (EPT) to map and write-protect critical kernel dispatch structure directly inside the processor. Any unauthorized modification attempts bypass guest hooks and trigger an immediate physical `VM-Exit`.
- **Sovereign Footprint (<10MB):** Utilizing Intel hardware pass-through execution, the guest operating system runs at 100% native hardware speed. The hypervisor remains completely passive and dormant unless an explicit safety constraint is violated, thus preserving older hardware resources with near-zero CPU overhead at rest.
- **Anti-Evasion Spoofing:** Intercepts and virtualizes hardware timing instructions to neutralize advanced malware attempting anti-VM evasion or sandbox detection routines.

## Directory Layout

The codebase is strictly structured into highly decoupled modules separating low-level state configuration from high-level interception logic:

```plaintext
└── project-hlidskjalf/
    ├── Cargo.toml            # Workspace orchestration and profile definitions
    ├── build.rs              # Kernel compilation glue for linking ntoskrnl/hal
    ├── src/
    │   ├── lib.rs            # Driver Entry Point and CPU feature discovery
    │   └── vmx/
    │       ├── mod.rs        # Submodule routing
    │       ├── init.rs       # VMXON initialization and CR4 manipulation
    │       ├── msr.rs        # MSR constants and VMX capability helpers
    │       ├── state.rs      # CPU state capture for guest/host VMCS fields
    │       ├── config.rs     # VMCS matrix building and host/guest setup
    │       ├── vmcs.rs       # Architecture-specific field encoding wrappers
    │       ├── ept.rs        # Multi-tiered identity page table mapping
    │       ├── memory.rs     # Static 4KB-aligned physical region structures
    │       ├── vmlaunch.rs   # Final boundary transition to Ring -1
    │       ├── context.rs    # General Purpose Register save state tracking
    │       ├── exit.rs       # Rust high-level event interception logic
    │       └── exit_asm.s    # Raw assembly context save and VMRESUME loop
    └── fuzz/                 # Automated verification harness for CPUID simulation
```

## Low-Level Dive

### 1. Driver Lifecycle & Subversion Entry

The engine compiles into a native Windows kernel driver (`.sys`). Upon loading, `DriverEntry` initiates a chain of low-level hardware transitions:

```rust
pub extern "system" fn DriverEntry(
    _driver_object: *mut DriverObject,
    _registry_path: *mut UnicodeString,
) -> i32 {
    unsafe {
        if enable_vmx().is_err() { return 0xC00000BBu32 as i32; }   // STATUS_NOT_SUPPORTED
        if setup_vmcs().is_err() { return 0xC00000BBu32 as i32; }   // STATUS_NOT_SUPPORTED
        vmlaunch();                                                 // Enter Ring -1 execution
    }
    0 // STATUS_SUCCESS
}
```

### 2. Physical VMX Initialization

Before taking control, the hypervisor checks the feature architecture using the native `CPUID` leaf 1 to confirm VMX bit 5 capability on ECX. It then executes a raw assembly transition into virtualization mode by modifying Control Register 4:

```rust
pub unsafe fn enable_vmx() -> Result<(), &'static str> {
    let mut cr4: u64;
    unsafe {
        asm!("mov {}, cr4", out(reg) cr4);
        cr4 |= 1 << 13; // Set CR4.VMXE to enable virtualization hardware
        asm!("mov cr4, {}", in(reg) cr4);
    }
    
    let pa = VMXON_REGION.get() as u64;
    unsafe {
        if !vmxon(pa) { return Err("VMXON failed"); }
    }
    Ok(())
}
```

### Extended Page Table (EPT) Identity Mapping

The hypervisor identity-maps the first **4 GB** of physical memory using **2 MB large pages**. Static footprint: 24 KB of page tables (1 PML4 + 1 PDPT + 4 PD pages).

```rust
pub unsafe fn identity_map_ept() {
    // PML4[0] -> PDPT -> 4 PD pages (one per GB)
    // Each PD entry is a 2 MB large page: physical | 0x87
}
```

Total static hypervisor memory (VMXON, VMCS, EPT, host stack) is capped at **~40 KB**.

### 4. Hardware Interception Loop (`VM-Exit`)

On VM-exit, control transfers to `vm_exit_wrapper` in `exit_asm.s`. The assembly saves GPRs, passes a `GuestContext` pointer in **RCX** (Windows x64 ABI), calls `vm_exit_handler_rust`, restores registers, and executes `vmresume`:

```asm
vm_exit_wrapper:
    push rax
    ; ... save remaining GPRs ...
    mov rcx, rsp
    sub rsp, 32
    call vm_exit_handler_rust
    add rsp, 32
    ; ... restore GPRs ...
    vmresume
```

## Build and Verification Automation

The infrastructure relies on strict, continuous evaluation policies verified inside isolated build spaces via GitHub Actions:

- **Compiler Matrix Verification (`build.yml`):** Linux unit tests plus Windows `x86_64-pc-windows-msvc` release build with Clippy `-D warnings`.
- **Static Analysis (`codeql.yml`):** Evaluates that unsafe Rust boundaries using automated semantic reasoning engines targeting extended security query sets.
- **Fuzzing Harness (`fuzz.yml`):** Utilizes LLVM libFuzzer via `cargo-fuzz` to feed randomized simulation data into low-level capability verification blocks, verifying panic resistance across unusual CPU layout paths.