use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

// Similiar to how we are using the println! macro for the vga

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // common address of the serial port
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    // SerialPort already implements Write so we don't have to
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

/// Prints to the host through the serial interface
#[macro_export] // means we don't need to import the macro in the main file
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, with an appended newline
#[macro_export] // means we don't need to import the macro in the main file
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\n")
    };
    ($fmt:expr) => {
        $crate::serial_print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*
    ));
}
