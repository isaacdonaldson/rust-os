#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
// needed for interrupts and exception handling
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buffer;

///////////////////////////////////
// Initialization

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

///////////////////////////////////
// Helpers

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/////////////////////////////////////////////
// Test logic

pub trait Testable {
    fn run(&self) -> ();
}

// Implements the Testable trait for all types T
// that implement the Fn() trait
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        // Uses the core library compiler type reflection to get the name of the type
        // For functions the name is the function name
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// Uses the Testable trait
// Usable by other tests
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run(); // Call the run function on the Testable trait
    }

    exit_qemu(QemuExitCode::Success);
}

// Usable by other tests
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop(); // Still needed as compiler cannot tell we exit before here
}

/////////////////////////////////////////////
// Entry Points
/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Tests also need the initializations
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/////////////////////////////////////////////
// Exits Qemu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
