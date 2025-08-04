.PHONY: clean clean-all build

build:
	cargo build --package crashpad-sys

clean:
	cargo clean

clean-all:
	rm -rf third_party
	cargo clean

rebuild: clean-all build