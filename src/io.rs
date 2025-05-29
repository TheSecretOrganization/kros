use core::arch::asm;

/// Writes a byte (`val`) to the specified I/O port (`port`).
///
/// # Safety
/// This function is `unsafe` because it performs raw hardware access.
/// Incorrect usage may cause undefined behavior or hardware faults.
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

/// Reads a byte from the specified I/O port and returns it.
///
/// # Safety
/// This function is `unsafe` because it performs raw hardware access.
/// Incorrect usage may cause undefined behavior or hardware faults.
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
