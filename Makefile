RFLAGS="-C link-arg=-s"

build: contract mock-ft mock-mft

contract: contracts/contract
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p contract --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/contract.wasm ./res/token_locker.wasm

release:
	$(call docker_build,_rust_setup.sh)
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/contract.wasm res/token_locker_release.wasm

mock-ft: contracts/mock-ft
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-ft --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_ft.wasm ./res/mock_ft.wasm

mock-mft: contracts/mock-mft
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-mft --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_mft.wasm ./res/mock_mft.wasm

unittest: build
	RUSTFLAGS=$(RFLAGS) cargo test --lib -- --nocapture

test: build
	RUSTFLAGS=$(RFLAGS) cargo test -- --nocapture

clean:
	cargo clean
	rm -rf res/

define docker_build
	docker build -t my-burrow-builder .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t my-burrow-builder \
		/bin/bash $(1)
endef