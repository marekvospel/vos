ENTRY(start)

SECTIONS {
  . = 1M;

  .boot :
  {
    *(.multiboot)
  }

  .text :
  {
    *(.text .text.*)
  }

  .rodata :
  {
    *(.rodata .rodata.*)
  }

  .data.rel.ro :
  {
    *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
  }

  .data :
  {
    *(.data .data.*)
  }

  .bss :
  {
    *(.bss .bss.*)
  }

}