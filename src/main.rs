#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod io;
#[unsafe(no_mangle)]
pub fn kmain() {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in b"Hello World!".iter().enumerate() {
        unsafe {
            core::ptr::write_volatile(vga_buffer.offset((i * 2) as isize), byte);
            core::ptr::write_volatile(vga_buffer.offset((i * 2 + 1) as isize), 0x0f);
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
