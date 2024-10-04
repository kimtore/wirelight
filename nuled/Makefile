build:
	cargo build --release

flash: build
	#espflash flash --monitor target/riscv32imc-esp-espidf/release/nuled
	espflash flash --monitor target/riscv32imc-unknown-none-elf/release/nuled
