#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod io;
mod pic;
mod vga_buffer;

#[unsafe(no_mangle)]
pub fn kmain() {
    println!("42");
    pic::remap(0x20, 0x28);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
