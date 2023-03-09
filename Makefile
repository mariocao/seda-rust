.PHONY: build build-wasm check clean fmt run run-build run-build-all run-build-wasm run-build-delegate test test-build wasm

MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
MKFILE_DIR := $(dir $(MKFILE_PATH))

ifeq ($(OS),Windows_NT)
	SEDA_BIN := seda.exe
	SEDA_DELEGATE_BIN := seda-delegate.exe
	SEDA_DEBUG_BIN := seda_debug.exe
else
	SEDA_BIN := seda
	SEDA_DELEGATE_BIN := seda-delegate
	SEDA_DEBUG_BIN := seda_debug
endif

SEDA_BIN_PATH := $(MKFILE_DIR)target/debug/$(SEDA_BIN)
SEDA_DELEGATE_BIN_PATH := $(MKFILE_DIR)target/debug/$(SEDA_DELEGATE_BIN)
SEDA_DEBUG_BIN_PATH := $(MKFILE_DIR)target/debug/$(SEDA_DEBUG_BIN)

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

# Runs cargo test excluding certain workspaces.
# Note the seda-debugger most be built for this command to work.
test:
	$(MAKE) start-test-rpc &
	cargo test --workspace --exclude cli --exclude consensus --exclude demo-cli --exclude promise-wasm-bin --exclude seda-cli --exclude seda-debugger
	$(MAKE) stop-test-rpc

# Builds the wasm binaries and then runs the same command as make test.
test-build: seda-debugger wasm-test
	cargo test --workspace --exclude cli --exclude consensus --exclude demo-cli --exclude promise-wasm-bin --exclude seda-cli --exclude seda-debugger

# Builds the wasm binaries.
wasm:
	$(foreach module, $(WASM_MODULES), cargo build -p $(module) --target wasm32-wasi;)

# Builds test wasm binaries.
wasm-test:
	$(foreach module, $(WASM_TEST_MODULES), cargo build -p $(module) --target wasm32-wasi;)

# Builds contracts wasm binaries.
build-contracts:
	cargo build -p seda-mainchain --target wasm32-unknown-unknown --release

# Make the seda debug-tools
seda-debugger:
	cargo build -p seda-debugger

# Just runs the prebuilt binary with the given args.
run-seda-debug:
	$(SEDA_DEBUG_BIN_PATH) $(RUN_ARGS)

# Runs the Test RPC Server
start-test-rpc:
	$(SEDA_DEBUG_BIN_PATH) test-rpc --addr 127.0.0.1:4657 start

# Stops the Test RPC Server
stop-test-rpc:
	$(SEDA_DEBUG_BIN_PATH) test-rpc --addr 127.0.0.1:4657 stop
