setup:
	cargo run --release --bin generate_dataset -- --count 1024 --output data/1k.dat
	cargo run --release --bin generate_dataset -- --count 1048576 --output data/1m.dat
	cargo run --release --bin generate_dataset -- --count 1073741824 --output data/1g.dat
