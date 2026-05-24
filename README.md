# Project Hliðskjálf

"Hliðskjálf... the high throne of the Allfather, from which all realms are watched."

Project Hliðskjálf is an open-source, ultra-lightweight Type-1.5 thin hypervisor. It is engineered to provide immutable, hardware-level security retrofits for legacy and End-of-Life (EOL) x86_64 operating systems, starting with Windows 10.

## The Motivation: A Stance Against Unnecessary Obsolescence

This project is a direct response to the macroeconomic vulnerability created by Microsoft’s decision to sunset Windows 10. Forcing users to migrate to Windows 11—a bloated, telemetry-heavy operating system that discards functional hardware—is not just an inconvenience; it is an unsustainable mandate.

Hliðskjálf empowers users to maintain control over their hardware. By dropping the host kernel into a hardware-enforced virtual machine sandbox at Ring -1, we invisible audit and police the operating system from above, rendering it secure without succumbing to forced platform upgrades.

## Architectural Pillars

- **Zero-Dependency Rust (`#![no_std]`):** Monitored completely within bare-metal Rust with zero reliance on standard libraries or OS runtimes, eliminating memory safety vulnerabilities at the foundational layer.
- **On-the-Fly Subversion:** Natively initializes Intel VT-x extensions via inline x86_64 assembly, dynamically virtualizing a live, running host kernel without requiring system reboots.
- **EPT Memory Shadowing:** Leverages Extended Page Tables (EPT) to write-protect critical kernel dispatch tables (SSDT/IDT) directly inside the processor silicon.
- **The Sovereign Footprint (<10MB):** Utilizing Intel pass-through execution, the host operating system runs at 100% native hardware speed.
- **Anti-Evasion Spoofing:** Intercepts and virtualizes hardware timing instructions (such as RDTSC) to neutralize advanced malware attempting anti-VM evasion or sandbox detection routines.

---

*Hliðskjálf – Watch over your realm, protect your legacy.*
