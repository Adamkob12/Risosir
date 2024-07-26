OUTPUT_ARCH( "riscv" )
ENTRY( maine )

SECTIONS
{
  . = 0x1000;

  .text : {
    *(.text .text.*)
    . = ALIGN(0x1000);
    PROVIDE(etext = .);
  }

  .rodata : {
    . = ALIGN(0x1000);
    *(.srodata .srodata.*)
    . = ALIGN(0x1000);
    *(.rodata .rodata.*)
  }

  .data : {
    . = ALIGN(0x1000);
    *(.sbss .sbss.*)
    . = ALIGN(0x1000);
    *(.bss .bss.*)
  }

  PROVIDE(end = .);
}
