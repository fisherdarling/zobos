build:
		rustup override set nightly
		cargo build --bin ZOBOS;
		@cp ./target/debug/ZOBOS ./ZOBOS;
		@chmod +x ./ZOBOS

clean:
		cargo clean
		rm -rf test/