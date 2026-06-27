.PHONY: check build limine iso run clean

check:
	cargo check -p novaspherex_kernel

build:
	cargo build --release -p novaspherex_kernel

limine:
	./tools/fetch-limine.sh

iso: build limine
	./tools/make-iso.sh

run: iso
	./tools/run-qemu.sh

clean:
	cargo clean
	rm -rf target/iso_root target/novaspherex.iso