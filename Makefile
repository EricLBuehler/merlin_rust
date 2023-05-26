run: $(file)
	cargo run -- $(file)

release:
	cargo build -r