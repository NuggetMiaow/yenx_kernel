TARGET_JSON := x86_64.json
KERNEL_BIN  := target/x86_64/release/yenx_kernel
ISO_FILE    := kernel.iso
QEMU        := qemu-system-x86_64
GRUB_PC_DIR := /usr/lib/grub/i386-pc

ifeq ($(wildcard $(GRUB_PC_DIR)/cdboot.img),)
$(error "cdboot.img not found. Install grub: sudo pacman -S grub")
endif

.PHONY: all kernel iso run clean

all: kernel

kernel:
	cargo build --target $(TARGET_JSON) --release

iso: kernel
	@mkdir -p iso/boot/grub
	cp $(KERNEL_BIN) iso/boot/yenx.bin
	echo 'set timeout=0' > iso/boot/grub/grub.cfg
	echo 'set default=0' >> iso/boot/grub/grub.cfg
	echo 'set gfxpayload=keep' >> iso/boot/grub/grub.cfg
	echo 'menuentry "Yenx Kernel" {' >> iso/boot/grub/grub.cfg
	echo '    multiboot2 /boot/yenx.bin' >> iso/boot/grub/grub.cfg
	echo '    boot' >> iso/boot/grub/grub.cfg
	echo '}' >> iso/boot/grub/grub.cfg
	# 关键：加上 iso9660 模块
	grub-mkimage -O i386-pc -o iso/boot/grub/core.img \
		-p /boot/grub \
		biosdisk part_msdos fat iso9660 multiboot2 normal configfile
	cat $(GRUB_PC_DIR)/cdboot.img iso/boot/grub/core.img > iso/eltorito.img
	xorriso -as mkisofs -R -J \
		-b eltorito.img -no-emul-boot -boot-load-size 4 -boot-info-table \
		-o $(ISO_FILE) iso/

run: iso
	$(QEMU) -cdrom $(ISO_FILE) -m 128M -cpu qemu64,+x2apic

clean:
	cargo clean
	rm -rf iso $(ISO_FILE) boot.o