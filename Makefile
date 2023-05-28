run: $(file)
	cargo run -- $(file)

release:
	cargo build -r --verbose
	cp target/release/merlin merlin

dev:
	cargo build
	cp target/debug/merlin merlin

test:
	cargo test --verbose

install:
	make release	
	cp merlin /usr/bin/merlin

backtrace: $(file)
	RUST_BACKTRACE=1 cargo run -- $(file)