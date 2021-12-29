#![no_std] // Do not import the stdlib implicitly
#![no_main] // Do not use normal rust entry chain (crt0 & main())

use core::panic::PanicInfo;

// Need to define a panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {} // Just spin loop forever for now
}

static HELLO: &[u8] = b"Hello World!";

#[no_mangle] // Keep the name after the compiler
pub extern "C" fn _start() -> ! {
    // We will exit and not return so `!` is appropriate
    // Use C calling convention & default entry point `_start`
    let vga_buff = 0xb8000 as *mut u8; // Cast into a raw pointer

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buff.offset(i as isize * 2) = byte; // Write byte in HELLO to offset of pointer
            *vga_buff.offset(i as isize * 2 + 1) = 0xb; // Set color to light cyan
        }
    }

    loop {}
}
