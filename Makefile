# CupruxOS build system

ARCH    ?= x86_64
TARGET  ?= $(ARCH)-unknown-none
QEMU    ?= qemu-system-$(ARCH)

KERNEL  = target/$(TARGET)/release/kernel

.PHONY: all build run clean fmt check

all: build

## Сборка ядра / Build kernel
build:
	cargo build --package cupruxos-kernel --release --target $(TARGET)

## Запуск в QEMU / Run in QEMU
run: build
	$(QEMU) \
		-m 256M \
		-serial stdio \
		-kernel $(KERNEL)

## Проверка кода / Lint
check:
	cargo clippy --package cupruxos-kernel --target $(TARGET)

## Форматирование / Format
fmt:
	cargo fmt --all

## Очистка / Clean
clean:
	cargo clean

## Помощь / Help
help:
	@echo "make build  — собрать ядро / build kernel"
	@echo "make run    — запустить в QEMU / run in QEMU"
	@echo "make check  — clippy lint"
	@echo "make fmt    — rustfmt"
	@echo "make ARCH=aarch64 build  — кросс-компиляция / cross-compile"
