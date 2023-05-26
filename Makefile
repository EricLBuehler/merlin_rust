run: $(file)
	cargo run -- $(file)

release:
	cargo build -r
	cp target/release/merlin merlin

dev:
	cargo build
	cp target/debug/merlin merlin