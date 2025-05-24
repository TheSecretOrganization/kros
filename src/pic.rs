const PIC_PARENT: u16 = 0x20;
const PIC_CHILD: u16 = 0xA0;
const PIC_PARENT_COMMAND: u16 = PIC_PARENT;
const PIC_PARENT_DATA: u16 = PIC_PARENT + 1;
const PIC_CHILD_COMMAND: u16 = PIC_CHILD;
const PIC_CHILD_DATA: u16 = PIC_CHILD + 1;
const PIC_EOI: u8 = 0x20;
const PIC_ICW1_INIT: u8 = 0x11;
const PIC_ICW4_8086: u8 = 0x01;

use crate::io::{inb, outb, wait};

fn outb_wait(port: u16, val: u8) {
    outb(port, val);
    wait();
}

#[allow(dead_code)]
pub fn eoi(irq: u8) {
    if irq >= 8 {
        outb(PIC_CHILD_COMMAND, PIC_EOI);
    }
    outb(PIC_PARENT_COMMAND, PIC_EOI);
}

fn get_port(irq: &mut u8) -> u16 {
    if *irq < 8 {
        PIC_PARENT_DATA
    } else {
        *irq -= 8;
        PIC_CHILD_DATA
    }
}

#[allow(dead_code)]
pub fn enable_irq(mut irq: u8) {
    let port = get_port(&mut irq);
    let mask = inb(port) & !(1 << irq);
    outb(port, mask);
}

#[allow(dead_code)]
pub fn disable_irq(mut irq: u8) {
    let port = get_port(&mut irq);
    let mask = inb(port) | (1 << irq);
    outb(port, mask);
}

pub fn remap(parent_offset: u8, child_offset: u8) {
    outb_wait(PIC_PARENT_COMMAND, PIC_ICW1_INIT);
    outb_wait(PIC_CHILD_COMMAND, PIC_ICW1_INIT);
    outb_wait(PIC_PARENT_DATA, parent_offset);
    outb_wait(PIC_CHILD_DATA, child_offset);
    outb_wait(PIC_PARENT_DATA, 4);
    outb_wait(PIC_CHILD_DATA, 2);
    outb_wait(PIC_PARENT_DATA, PIC_ICW4_8086);
    outb_wait(PIC_CHILD_DATA, PIC_ICW4_8086);

    // Disable all IRQs
    outb_wait(PIC_PARENT_DATA, 255);
    outb_wait(PIC_CHILD_DATA, 255);
}
