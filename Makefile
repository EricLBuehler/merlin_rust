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
	cargo test trc_test --verbose -- --nocapture

install:
	make release	
	cp merlin /usr/bin/merlin

backtrace: $(file)
	RUST_BACKTRACE=1 cargo run -- $(file)

clean:
	cargo clean
	rm merlin

fmt:
	cargo clippy --fix --allow-dirty
	cargo fmt
	
doc:
	cargo doc --no-deps --open