# GDB configuration for STM32F103C8T6 (Blue Pill)
target extended-remote :3333

# Print demangled symbols by default
set print asm-demangle on

# Detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

# STM32F103 specific settings
monitor reset halt
monitor flash probe 0

# Run the program
monitor arm semihosting enable
load
continue