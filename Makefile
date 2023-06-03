build:
	cargo build --release
	mv target/release/pot ./pot
	rm -rf target/
