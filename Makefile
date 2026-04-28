STELLAR_CLI := "/path/to/target/debug/stellar"

default: build

test: build
	cargo test --all --tests

spec-test: build-manual build
	cargo test -p test-suites
	cargo test -p test-suites --test test_liquidation -- --nocapture
	ls -l target/wasm32v1-none/release/*.wasm

build-manual:
	SOROBAN_SDK_BUILD_SYSTEM_SUPPORTS_SPEC_SHAKING_V2=1 cargo rustc --manifest-path=pool-factory/Cargo.toml --crate-type=cdylib --target=wasm32v1-none --release
	SOROBAN_SDK_BUILD_SYSTEM_SUPPORTS_SPEC_SHAKING_V2=1 cargo rustc --manifest-path=backstop/Cargo.toml --crate-type=cdylib --target=wasm32v1-none --release
	SOROBAN_SDK_BUILD_SYSTEM_SUPPORTS_SPEC_SHAKING_V2=1 cargo rustc --manifest-path=pool/Cargo.toml --crate-type=cdylib --target=wasm32v1-none --release
	mv target/wasm32v1-none/release/pool_factory.wasm target/wasm32v1-none/release/_manual_pool_factory.wasm
	mv target/wasm32v1-none/release/backstop.wasm target/wasm32v1-none/release/_manual_backstop.wasm
	mv target/wasm32v1-none/release/pool.wasm target/wasm32v1-none/release/_manual_pool.wasm

build:
	$(STELLAR_CLI) contract build --package pool-factory --optimize
	$(STELLAR_CLI) contract build --package backstop --optimize
	$(STELLAR_CLI) contract build --package pool --optimize

fmt:
	cargo fmt --all

clean:
	cargo clean

generate-js:
	stellar contract bindings typescript --overwrite \
		--contract-id CBWH54OKUK6U2J2A4J2REJEYB625NEFCHISWXLOPR2D2D6FTN63TJTWN \
		--wasm ./target/wasm32v1-none/release/backstop.wasm --output-dir ./js/js-backstop/ \
		--rpc-url http://localhost:8000 --network-passphrase "Standalone Network ; February 2017" --network Standalone
	stellar contract bindings typescript --overwrite \
		--contract-id CBWH54OKUK6U2J2A4J2REJEYB625NEFCHISWXLOPR2D2D6FTN63TJTWN \
		--wasm ./target/wasm32v1-none/release/pool_factory.wasm --output-dir ./js/js-pool-factory/ \
		--rpc-url http://localhost:8000 --network-passphrase "Standalone Network ; February 2017" --network Standalone
	stellar contract bindings typescript --overwrite \
		--contract-id CBWH54OKUK6U2J2A4J2REJEYB625NEFCHISWXLOPR2D2D6FTN63TJTWN \
		--wasm ./target/wasm32v1-none/release/pool.wasm --output-dir ./js/js-pool/ \
		--rpc-url http://localhost:8000 --network-passphrase "Standalone Network ; February 2017" --network Standalone
