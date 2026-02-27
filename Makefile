# CupruxOS build system

ARCH    ?= x86_64
TARGET  ?= $(ARCH)-unknown-none
QEMU    ?= qemu-system-$(ARCH)
KERNEL   = target/$(TARGET)/release/kernel
ISO      = cupruxos.iso

.PHONY: all build iso run clean fmt check

all: build

## Сборка ядра / Build kernel
build:
	cargo build --package cupruxos-kernel --release --target $(TARGET)

## Создать ISO образ / Create ISO image
iso: build
	mkdir -p iso_root/boot
	cp $(KERNEL) iso_root/boot/cupruxos-kernel
	xorriso -as mkisofs \
		-b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image \
		--protective-msdos-label \
		iso_root -o $(ISO)
	@echo "ISO создан / ISO created: $(ISO)"

## Запуск в QEMU / Run in QEMU
run: iso
	$(QEMU) \
		-m 256M \
		-cdrom $(ISO) \
		-serial stdio \
		-no-reboot \
		-no-shutdown

## Запуск без графики (только UART) / Run headless (UART only)
run-headless: iso
	$(QEMU) \
		-m 256M \
		-cdrom $(ISO) \
		-serial stdio \
		-display none \
		-no-reboot

## Проверка кода / Lint
check:
	cargo clippy --package cupruxos-kernel --target $(TARGET)

## Форматирование / Format
fmt:
	cargo fmt --all

## Очистка / Clean
clean:
	cargo clean
	rm -f $(ISO)
	rm -rf iso_root/boot/cupruxos-kernel

## Помощь / Help
help:
	@echo "make build        — собрать ядро / build kernel"
	@echo "make iso          — создать ISO  / create ISO"
	@echo "make run          — запустить в QEMU / run in QEMU"
	@echo "make run-headless — только UART вывод / UART only"
	@echo "make check        — clippy lint"
	@echo "make fmt          — rustfmt"
	@echo "make ARCH=aarch64 build — кросс-компиляция"
