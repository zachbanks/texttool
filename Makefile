# Convenience tasks for developing texttool.
.PHONY: all fmt fmt-check lint test check build install setup release clean

all: check

## fmt: format the source tree with rustfmt
fmt:
	cargo fmt

## fmt-check: verify formatting without modifying files
fmt-check:
	cargo fmt --check

## lint: run clippy, treating warnings as errors
lint:
	cargo clippy --all-targets -- -D warnings

## test: run the unit test suite
test:
	cargo test

## check: format check + lint + test (run this before committing)
check: fmt-check lint test

## build: build an optimized release binary
build:
	cargo build --release

## install: build and copy the binary to ~/.local/bin
install:
	./scripts/install.sh

## setup: enable the auto-bump / auto-install git hooks
setup:
	./scripts/setup-hooks.sh

## release: cut an exact version + tag, e.g. `make release VERSION=0.2.0`
release:
	@test -n "$(VERSION)" || { echo "usage: make release VERSION=x.y.z"; exit 1; }
	./scripts/release.sh "$(VERSION)"

## clean: remove build artifacts
clean:
	cargo clean
