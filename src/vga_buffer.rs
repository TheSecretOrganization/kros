use crate::io::outb;
use core::{
    fmt,
    ptr::{read_volatile, write_volatile},
};
use lazy_static::lazy_static;
use spin::Mutex;

const DEFAULT_FOREGROUND: Color = Color::LightBlue;
const DEFAULT_BACKGROUND: Color = Color::Black;
const BUFFER_ADDRESS: usize = 0xb8000;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const BUFFER_CMD_PORT: u16 = 0x3D4;
const BUFFER_DATA_PORT: u16 = 0x3D5;
const CURSOR_LOW: u8 = 0x0F;
const CURSOR_HIGH: u8 = 0x0E;
const BLANK: u8 = b' ';
const UNPRINTABLE: u8 = b'*';

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    row_position: usize,
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    fn read_byte_at(&mut self, row: usize, col: usize) -> u8 {
        unsafe { read_volatile(&self.buffer.chars[row][col]).ascii_character }
    }

    fn move_cursor_at(&mut self, row: usize, col: usize) {
        let pos: u16 = (row * BUFFER_WIDTH + col) as u16;

        outb(BUFFER_CMD_PORT, CURSOR_LOW);
        outb(BUFFER_DATA_PORT, (pos & 0xFF) as u8);
        outb(BUFFER_CMD_PORT, CURSOR_HIGH);
        outb(BUFFER_DATA_PORT, ((pos >> 8) & 0xFF) as u8);
    }

    fn move_cursor(&mut self) {
        self.move_cursor_at(self.row_position, self.column_position);
    }

    fn write_byte_at(&mut self, byte: u8, row: usize, col: usize) {
        let color_code = self.color_code;
        unsafe {
            write_volatile(
                &mut self.buffer.chars[row][col],
                ScreenChar {
                    ascii_character: byte,
                    color_code,
                },
            );
        }
    }

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

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    fn new_line(&mut self) {
        self.column_position = 0;
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            self.row_position = BUFFER_HEIGHT - 1;
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.read_byte_at(row, col);
                    self.write_byte_at(character, row - 1, col);
                }
            }
        }
    }

    fn trim_back(&mut self) {
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

    #[allow(dead_code)]
    pub fn delete_byte(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            self.write_byte_at(BLANK, self.row_position, self.column_position);
        } else {
            self.trim_back();
        }
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.write_byte_at(BLANK, row, col);
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.row_position = 0;
        self.column_position = 0;
        self.move_cursor();
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(DEFAULT_FOREGROUND, DEFAULT_BACKGROUND),
        buffer: unsafe { &mut *(BUFFER_ADDRESS as *mut Buffer) },
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! clear_screen {
    () => {
        $crate::vga_buffer::WRITER.lock().clear_screen()
    };
}
