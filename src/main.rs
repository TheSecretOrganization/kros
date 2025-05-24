#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in b"Hello World!".iter().enumerate() {
        unsafe {
            core::ptr::write_volatile(vga_buffer.add(i * 2), byte);
            core::ptr::write_volatile(vga_buffer.add(i * 2 + 1), 0x0f);
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
