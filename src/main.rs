#![no_std] // Do not import the stdlib implicitly
#![no_main] // Do not use normal rust entry chain (crt0 & main())
// TEST
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::println;
use core::panic::PanicInfo;

// Need to define a panic handler
// Conditional compilation for handler not in test
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {} // Print panic info and loop forever
}

// Need to define a panic handler
// Conditional compilation for test cases
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

/////////////////////////////////////////////
// Main entry point for the kernel
#[no_mangle] // Keep the name after the compiler
pub extern "C" fn _start() -> ! {
    // We will exit and not return so `!` is appropriate
    // Use C calling convention & default entry point `_start`

    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    // panic!("at the disco");

    loop {}
}
