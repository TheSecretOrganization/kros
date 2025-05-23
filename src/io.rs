use core::arch::asm;

#[inline(always)]
pub fn outb(port: u16, val: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("al") val,
            in("dx") port,
            options(nomem, nostack, preserves_flags),
        );
    }
}

#[allow(dead_code)]
#[inline(always)]
pub fn inb(port: u16) -> u8 {
    let mut ret: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") ret,
            in("dx") port,
            options(nomem, nostack, preserves_flags),
        );
    }
    ret
}
