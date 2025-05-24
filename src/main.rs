#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod io;
mod vga_buffer;

#[unsafe(no_mangle)]
pub fn kmain() {
    println!("42");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
