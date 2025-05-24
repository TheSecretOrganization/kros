ISO_DIR     := iso
BOOT_DIR	:= $(ISO_DIR)/boot
GRUB_DIR	:= $(BOOT_DIR)/grub
SRC_DIR		:= src
NAME        := kros
GRUB_CFG    := grub.cfg
TARGET 		:= i686-unknown-none
ELF         := target/$(TARGET)/debug/$(NAME)
ISO         := $(NAME).iso
RUST_SRC	:= $(shell find . -type f -name '*.rs')
OBJ 		:= $(shell find $(SRC_DIR) -type f -name '*.o')

.PHONY: all iso build re run clean check-format format

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

check-format:
	@rustfmt --check $(RUST_SRC) || (echo "Formatting issues detected! Run 'make format' to fix." && exit 1)

format:
	@echo "Formatting source code..."
	@rustfmt $(RUST_SRC)
