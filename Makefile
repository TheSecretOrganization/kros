ISO_DIR     := iso
BOOT_DIR	:= $(ISO_DIR)/boot
GRUB_DIR	:= $(BOOT_DIR)/grub
SRC_DIR		:= src
NAME        := kros
GRUB_CFG    := grub.cfg
TARGET 		:= i686-kros
ELF         := target/$(TARGET)/debug/$(NAME)
ISO         := $(NAME).iso
OBJ 		:= $(shell find $(SRC_DIR) -type f -name '*.o')

ifeq ($(shell uname -s), Darwin)
    GRUB_MK := i686-elf-grub-mkrescue
else
    GRUB_MK := grub-mkrescue
endif

.PHONY: all iso build re run clean

all: iso

iso: $(GRUB_CFG) build
	mkdir -p $(GRUB_DIR)
	cp $(GRUB_CFG) $(GRUB_DIR)
	cp $(ELF) $(BOOT_DIR)/$(NAME).elf
	$(GRUB_MK) -o $(ISO) $(ISO_DIR)

build:
	cargo build

re: clean
	$(MAKE) all

run: all
	qemu-system-i386 -cdrom $(ISO) -no-reboot

clean:
	cargo clean
	rm -rf $(ISO) $(ISO_DIR) $(OBJ)
