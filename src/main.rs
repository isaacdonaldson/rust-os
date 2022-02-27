#![no_std] // Do not import the stdlib implicitly
#![no_main] // Do not use normal rust entry chain (crt0 & main())

use core::panic::PanicInfo;

mod vga_buffer;

// Need to define a panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {} // Print panic info and loop forever
}

#[no_mangle] // Keep the name after the compiler
pub extern "C" fn _start() -> ! {
    // We will exit and not return so `!` is appropriate
    // Use C calling convention & default entry point `_start`

    println!("The numbers are {} and {}", 42, 1.0 / 3.0);
    println!("\n\n");
    panic!("Some panic message");

    loop {}
}
