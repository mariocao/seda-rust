.PHONY: build build-wasm check clean fmt run run-build run-build-all run-build-wasm run-build-delegate test test-build wasm

MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
MKFILE_DIR := $(dir $(MKFILE_PATH))

ifeq ($(OS),Windows_NT)
	SEDA_BIN := seda.exe
else
	SEDA_BIN := seda
endif

ifeq ($(OS),Windows_NT)
	SEDA_DELEGATE_BIN := seda-delegate.exe
else
	SEDA_DELEGATE_BIN := seda-delegate
endif

SEDA_BIN_PATH := $(MKFILE_DIR)target/debug/$(SEDA_BIN)
SEDA_DELEGATE_BIN_PATH := $(MKFILE_DIR)target/debug/$(SEDA_DELEGATE_BIN)

WASM_MODULES := $(notdir $(filter-out $(MKFILE_DIR)wasm/test,$(wildcard $(MKFILE_DIR)wasm/*)))
WASM_TEST_MODULES := $(notdir $(wildcard $(MKFILE_DIR)wasm/test/*))

# Builds only the seda binary.
build:
	cargo build

# Builds the wasm binaries and the seda binary.
build-wasm: wasm
	cargo build

# Runs clippy with the deny warnings flag.
check:
	RUSTFLAGS="-D warnings" cargo clippy --all-features

# Runs cargo clean.
clean:
	cargo clean

# Runs cargo +nightly fmt --all.
fmt:
	cargo +nightly fmt --all

# If the first argument is "run"...
ifneq (,$(findstring run,$(firstword $(MAKECMDGOALS))))
  # use the rest as arguments for "run"
  RUN_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  # ...and turn them into do-nothing targets
  $(eval $(RUN_ARGS):;@:)
endif

# Just runs the prebuilt binary with the given args.
run:
	$(SEDA_BIN_PATH) $(RUN_ARGS)

# Builds only seda-before running with the given args.
run-build: build
	$(SEDA_BIN_PATH) $(RUN_ARGS)

# Builds everything before running with the given args.
run-build-all: build-wasm
	$(SEDA_BIN_PATH) $(RUN_ARGS)

# Builds only the wasm's before re-running with the given args.
run-build-wasm: wasm
	$(SEDA_BIN_PATH) $(RUN_ARGS)

# Builds only seda before executing the delegation binary
run-build-delegate: build
	$(SEDA_DELEGATE_BIN_PATH) $(RUN_ARGS)


# Runs cargo test --workspace --exclude demo-cli --exclude seda-cli --exclude promise-wasm-bin.
test:
	# $($(MAKE) start-test-rpc)
	cargo test --workspace --exclude demo-cli --exclude seda-cli --exclude promise-wasm-bin --exclude seda-delegate-cli
	# $(MAKE) stop-test-rpc

# Builds the wasm binaries and then runs the same command as make test.
test-build: wasm-test build-contracts
	cargo test --workspace --exclude demo-cli --exclude seda-cli --exclude promise-wasm-bin --exclude seda-delegate-cli

# Builds the wasm binaries.
wasm:
	$(foreach module, $(WASM_MODULES), cargo build -p $(module) --target wasm32-wasi;)

# Builds test wasm binaries.
wasm-test:
	$(foreach module, $(WASM_TEST_MODULES), cargo build -p $(module) --target wasm32-wasi;)

# Builds contracts wasm binaries.
build-contracts:
	cargo build -p seda-mainchain --target wasm32-unknown-unknown --release

# Runs the Test RPC Server
start-test-rpc:
	$(SEDA_BIN_PATH) debug-mode test-rpc start

# Stops the Test RPC Server
stop-test-rpc:
	$(SEDA_BIN_PATH) debug-mode test-rpc stop
