target extended-remote :3333

# Print demangled symbols by default
set print asm-demangle on

# Detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

# Run the program
monitor arm semihosting enable
load
continue