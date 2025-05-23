ISO_DIR     := iso
BOOT_DIR	:= $(ISO_DIR)/boot
GRUB_DIR	:= $(BOOT_DIR)/grub
SRC_DIR		:= src
NAME        := kros
GRUB_CFG    := grub.cfg
TARGET 		:= i686-unknown-none
ELF         := target/$(TARGET)/debug/$(NAME)
ISO         := $(NAME).iso
BIN			:= $(NAME).bin
OBJ 		:= $(shell find $(SRC_DIR) -type f -name '*.o')

.PHONY: all re run clean

all: $(ISO)

$(ISO): $(GRUB_CFG) $(BIN)
	mkdir -p $(GRUB_DIR)
	cp $(GRUB_CFG) $(GRUB_DIR)
	cp $(ELF) $(BOOT_DIR)/$(NAME).elf
	grub-mkrescue -o $(ISO) $(ISO_DIR)

$(BIN):
	cargo build

re: clean
	$(MAKE) all

run: all
	qemu-system-i386 -cdrom $(ISO) -no-reboot

clean:
	cargo clean
	rm -rf $(BIN) $(ISO) $(ISO_DIR) $(OBJ)
