build run:
	cargo $@
debug:
	RUST_LOG=notice cargo run -- --debug
test:
	RUST_LOG=notice cargo $@
simple fancy jpg:
	RUST_LOG=debug cargo run -- --debug --test=$@
