#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod io;
mod pic;
mod spin;
mod vga;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    clear_screen!();
    println!("42");
    pic::remap(0x20, 0x28);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{:?}", info);
    loop {}
}
