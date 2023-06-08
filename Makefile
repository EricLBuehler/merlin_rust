run: $(file)
	cargo run -- $(file)

release:
	cargo build -r --verbose
	cp target/release/merlin merlin

dev:
	cargo build
	cp target/debug/merlin merlin

test:
	cargo test merlin_tests --verbose
	cargo test trc_tests --verbose

install:
	make release	
	cp merlin /usr/bin/merlin

backtrace: $(file)
	RUST_BACKTRACE=1 cargo run -- $(file)

clean:
	cargo clean
	rm merlin

fmt:
	cargo clippy --fix
	cargo fmt