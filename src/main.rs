#![no_std] // Do not import the stdlib implicitly
#![no_main] // Do not use normal rust entry chain (crt0 & main())
// TEST
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::println;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

// Need to define a panic handler
// Conditional compilation for handler not in test
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop(); // Print panic info and loop forever
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

// providing a type checked signature for the
// bootloader crate to call
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // We will exit and not return so `!` is appropriate
    // Use C calling convention & default entry point `_start`
    use blog_os::memory;
    use blog_os::memory::BootInfoFrameAllocator;
    use x86_64::{structures::paging::Page, VirtAddr};

    println!("Hello World{}", "!");

    // initialize the kernel
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    // choose address that needs a new frame allocated
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    // use allocator that can actually allocate a new frame
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)]
    test_main();

    println!("The kernel did not crash and reaches this point");

    // panic!("at the disco");

    blog_os::hlt_loop();
}
