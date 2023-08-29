# Vos
Vos is a custom OS created to learn more about what kernels do and about as
many low level things as possible.

## Boot error codes

| Code | Message                               |
|------|---------------------------------------|
|  M   | Not booted with multiboot             |
|  C   | CPUID instruction is not supported    |
|  L   | Long mode is not supported by the cpu |

## Atributions / Resources
- [Blog OS](https://github.com/phil-opp/blog_os) - A blog explaining many things from the boot process to memory management. [MIT/Apache-2.0]
- [Building an OS](https://www.youtube.com/playlist?list=PLFjM7v6KGMpiH2G-kT781ByCNC_0pKpPN) - A video series explaining everything including the bootloader, drivers etc. [CC-BY-SA-4.0]
- [OSDev Wiki](https://wiki.osdev.org) - A wiki about stuff like VGA text mode etc. includes code examples both in C and asm. [CC0]