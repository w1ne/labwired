MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 128K
  RAM : ORIGIN = 0x20000000, LENGTH = 16K
}

SECTIONS
{
  .vector_table 0x00000000 :
  {
    LONG(0x20004000); /* Initial SP */
    LONG(Reset + 1);  /* Reset Handler (Thumb) */
  } > FLASH

  .text :
  {
    *(.text*)
  } > FLASH

  /DISCARD/ :
  {
    *(.ARM.exidx*)
    *(.note.gnu.build-id*)
  }
}
