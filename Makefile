ISO_DIR     := iso
BOOT_DIR	:= $(ISO_DIR)/boot
GRUB_DIR	:= $(BOOT_DIR)/grub
SRC_DIR		:= src
NAME        := kros
GRUB_CFG    := grub.cfg
TARGET 		:= i686-unknown-none
ELF         := target/$(TARGET)/debug/$(NAME)
ISO         := $(NAME).iso
OBJ 		:= $(shell find $(SRC_DIR) -type f -name '*.o')


all: iso

iso: $(GRUB_CFG) build
	@echo "Building $(ISO)..."
	@mkdir -p $(GRUB_DIR)
	@cp $(GRUB_CFG) $(GRUB_DIR)
	@cp $(ELF) $(BOOT_DIR)/$(NAME).elf
	@grub-mkrescue -o $(ISO) $(ISO_DIR)

build:
	@echo "Building $(ELF)..."
	@cargo build

re: clean
	@$(MAKE) all

run: all
	@echo "Starting $(NAME)..."
	@qemu-system-i386 -cdrom $(ISO) -no-reboot

clean:
	@echo "Cleaning repository..."
	@cargo clean
	@rm -rf $(ISO) $(ISO_DIR) $(OBJ)
