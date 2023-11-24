arch		?=	x86_64
target	?=	$(arch)-custom
kernel	:=	build/kernel-$(arch).bin
iso			:=	build/image-$(arch).iso
rust_kernel	:=target/$(target)/release/libkernel.a

ld_script	:=	arch/$(arch)/linker.ld
grub_cfg	:=	arch/$(arch)/grub.cfg

asm_src		:=	$(wildcard arch/$(arch)/*.asm)
asm_obj		:=	$(patsubst arch/$(arch)/%.asm, build/arch/$(arch)/%.o, $(asm_src))

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@rm -rf build

$(kernel): kernel	$(asm_obj)	$(ld_script)
	@ld -n -T $(ld_script) -o $(kernel) $(asm_obj) $(rust_kernel)

kernel:
	@RUST_TARGET_PATH=$(shell pwd)/targets cargo build --target $(target) --release

build/arch/$(arch)/%.o:	arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -f elf64 $< -o $@

# ISO files
iso:	$(iso)

$(iso):	$(kernel)	$(grub_cfg)
	@mkdir -p build/iso/boot/grub
	@cp $(kernel) build/iso/boot/vos-kernel
	@cp $(grub_cfg) build/iso/boot/grub/
	@grub-mkrescue -o $(iso) build/iso
	@rm -r build/iso

run:	$(iso)
	@qemu-system-x86_64 -cdrom $(iso) -enable-kvm -serial stdio
