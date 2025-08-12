/* Memory layout for STM32F103C8T6 (Blue Pill) */
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* STM32F103C8T6 has 64K flash, 20K RAM */
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM : ORIGIN = 0x20000000, LENGTH = 20K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);