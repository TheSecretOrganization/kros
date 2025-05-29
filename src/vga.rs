use crate::io::outb;
use crate::spin::Spinlock;
use core::{
    fmt,
    ptr::{read_volatile, write_volatile},
};

const BUFFER_ADDRESS: usize = 0xb8000;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const BUFFER_CMD_PORT: u16 = 0x3D4;
const BUFFER_DATA_PORT: u16 = 0x3D5;
const CURSOR_LOW: u8 = 0x0F;
const CURSOR_HIGH: u8 = 0x0E;
const BLANK: u8 = b' ';
const UNPRINTABLE: u8 = b'*';

/// Global VGA text writer protected by a spinlock.
///
/// This static allows safe concurrent access to VGA text buffer from multiple threads.
pub static WRITER: Spinlock<Writer> = Spinlock::new(Writer {
    row_position: 0,
    column_position: 0,
    color_code: ColorCode::default(),
});

/// Print text to the screen.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

/// Print to the screen with a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Clear the screen buffer.
#[macro_export]
macro_rules! clear_screen {
    () => {
        $crate::vga::WRITER.lock().clear_screen()
    };
}

/// Internal function used by `print!` and `println!` macros to write formatted text.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

/// VGA text mode color codes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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

/// A combined foreground/background VGA color code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Creates a new color code.
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    /// Default color: light blue text on black background.
    const fn default() -> ColorCode {
        ColorCode::new(Color::LightBlue, Color::Black)
    }
}

/// A single character and its associated color on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    const fn default() -> Self {
        ScreenChar {
            ascii_character: b' ',
            color_code: ColorCode::default(),
        }
    }
}

/// VGA text buffer, a 2D grid of `ScreenChar`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    /// Creates a blank buffer (compile-time only, not used at runtime).
    #[allow(dead_code)]
    pub const fn new() -> Buffer {
        let chars = [[ScreenChar::default(); BUFFER_WIDTH]; BUFFER_HEIGHT];
        Buffer { chars }
    }

    /// Returns a mutable reference to the VGA buffer in memory.
    pub fn vga() -> &'static mut Buffer {
        unsafe { &mut *(BUFFER_ADDRESS as *mut Buffer) }
    }
}

/// Text writer for VGA memory. Manages cursor position and writing.
pub struct Writer {
    row_position: usize,
    column_position: usize,
    color_code: ColorCode,
}

impl Writer {
    /// Reads the byte at the given screen position (volatile read).
    fn read_byte_at(&mut self, row: usize, col: usize) -> u8 {
        unsafe { read_volatile(&Buffer::vga().chars[row][col]).ascii_character }
    }

    /// Moves the VGA hardware cursor to a specific screen position.
    fn move_cursor_at(&mut self, row: usize, col: usize) {
        let pos: u16 = (row * BUFFER_WIDTH + col) as u16;

        outb(BUFFER_CMD_PORT, CURSOR_LOW);
        outb(BUFFER_DATA_PORT, (pos & 0xFF) as u8);
        outb(BUFFER_CMD_PORT, CURSOR_HIGH);
        outb(BUFFER_DATA_PORT, ((pos >> 8) & 0xFF) as u8);
    }

    /// Moves the VGA hardware cursor to the current logical position.
    fn move_cursor(&mut self) {
        self.move_cursor_at(self.row_position, self.column_position);
    }

    /// Writes a single byte to the specified position.
    fn write_byte_at(&mut self, byte: u8, row: usize, col: usize) {
        let color_code = self.color_code;
        unsafe {
            write_volatile(
                &mut Buffer::vga().chars[row][col],
                ScreenChar {
                    ascii_character: byte,
                    color_code,
                },
            );
        }
    }

    /// Writes a single byte, handling newlines and character wrapping.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line();
                self.move_cursor();
            }
            b' '..=b'~' => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                self.write_byte_at(byte, self.row_position, self.column_position);
                self.column_position += 1;
                self.move_cursor();
            }
            _ => self.write_byte(UNPRINTABLE),
        }
    }

    /// Writes an entire string to the screen.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    /// Moves to a new line, scrolling if necessary.
    fn new_line(&mut self) {
        self.column_position = 0;
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            self.row_position = BUFFER_HEIGHT - 1;
            unsafe {
                let buffer_ptr = BUFFER_ADDRESS as *mut ScreenChar;
                core::ptr::copy(
                    buffer_ptr.add(BUFFER_WIDTH),
                    buffer_ptr,
                    BUFFER_WIDTH * (BUFFER_HEIGHT - 1),
                );
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        }
    }

    /// Moves the cursor back to the previous non-blank character.
    fn move_to_previous_non_blank(&mut self) {
        while self.row_position > 0 {
            self.row_position -= 1;
            self.column_position = BUFFER_WIDTH - 1;
            while self.column_position > 0 {
                let character = self.read_byte_at(self.row_position, self.column_position - 1);
                if character != BLANK {
                    return;
                }
                self.column_position -= 1;
            }
        }
    }

    /// Deletes the last character from the screen.
    #[allow(dead_code)]
    pub fn delete_byte(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            self.write_byte_at(BLANK, self.row_position, self.column_position);
        } else {
            self.move_to_previous_non_blank();
        }
        self.move_cursor();
    }

    /// Clears a specific row.
    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.write_byte_at(BLANK, row, col);
        }
    }

    /// Clears the entire screen buffer.
    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.row_position = 0;
        self.column_position = 0;
        self.move_cursor();
    }
}

/// Implements the core `fmt::Write` trait, allowing use with `write!()` macros.
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
