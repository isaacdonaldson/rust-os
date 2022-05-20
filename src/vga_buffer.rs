use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // copyable, printable, and comparable
#[repr(u8)] // stores each variant as a u8
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // ABI is gonna be the same as the underlying type
struct ColorCode(u8); // Type alias to u8 containing both foreground and background color

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        // first 4 bits are background, last 4 are foreground
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // Makes the struct behave the same way a C struct would
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)] // same ABI layout as its single field
struct Buffer {
    // VGA buffer is stored in a 2D array of 25 wide, 80 tall
    // Volatile here tells compiler that the read/write operations have side effects
    // So compiler will not optimize away the read/write operations
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer, // Buffer is 'static because it is around for the whole program
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        // Print a string byte by byte
        for byte in s.bytes() {
            match byte {
                // If in the ASCII range or newline, print char
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Not in the ASCII range, print a solid square
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    // If we are at the end of the line, we should wrap
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1; // write to last row
                let col = self.column_position; // current position to write to

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                // Gets the character and shifts it up a row
                // Here row 0 is the top of the screen so we do not include it
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1); // Clears the top row
        self.column_position = 0; // Set position back to start of line
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            // Overwrites every character in the row with a blank
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Rust has problem running some code statically (at compile time)
// Lazy_static allows the code to be defined the firs time it is called (runtime)
// But can still behave like it is static
lazy_static! {
    // Since OS has no support for blocking or threads, we use a spin lock mutex
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// $crate here allows us to not have to import in other files
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Need to use fmt crate to get access to write_fmt method
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // disables interrupts while printing to avoid deadlocks (if interrupts prints)
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

///////////////////////////////////
// Tests
#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string the fits on a single line";

    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");

        for (i, c) in s.chars().enumerate() {
            // BUFFER_HEIGHT - 2 because it would print the \n character
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
