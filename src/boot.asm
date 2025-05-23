MAGIC		equ 0x1BADB002
FLAGS		equ 1<<0 | 1<<1
CHECKSUM	equ -(MAGIC + FLAGS)

section .multiboot
align 4
dd MAGIC
dd FLAGS
dd CHECKSUM

section .bss
align 16
stack_bottom:
	resb 16384
stack_top:

section .text
global _start
extern kmain

_start:
	mov esp, stack_top
	call kmain
	; sti
loop:
	jmp loop
