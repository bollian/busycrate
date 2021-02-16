PREFIX ?= /usr/local

target/release/busycrate:
	cargo build --release

install: target/release/busycrate
	cp $< $(PREFIX)/bin

.PHONY: install
