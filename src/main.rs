#![no_std] // Do not import the stdlib implicitly
#![no_main] // Do not use normal rust entry chain (crt0 & main())
// TEST
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

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
    use blog_os::allocator;
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");

    // initialize the kernel
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&&boot_info.memory_map) };

    // Heap allocation
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let ref_counted = Rc::new(vec![1, 2, 3]);
    let cloned_ref = ref_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_ref)
    );
    core::mem::drop(ref_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_ref));

    #[cfg(test)]
    test_main();

    println!("The kernel did not crash and reaches this point");

    // panic!("at the disco");

    blog_os::hlt_loop();
}
