.PHONY: clean build

build:
	cargo build --package crashpad-sys

clean:
	rm -rf third_party
	cargo clean

rebuild: clean build