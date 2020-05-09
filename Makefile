build:
		rustup override set nightly-2020-05-08
		cargo build --bin ZOBOS;
		@cp ./target/debug/ZOBOS ./ZOBOS;
		@chmod +x ./ZOBOS

clean:
		cargo clean
		rm -rf test/