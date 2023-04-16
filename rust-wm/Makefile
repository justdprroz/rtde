all: prod

prod:
	cargo build

install: prod
	cp -f ./target/release/rust-wm /usr/local/bin

dev:
	cargo build --debug
	cp -f ./target/build/rust-wm /usr/local/bin
