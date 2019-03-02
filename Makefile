
.PHONY: install-deps
install-deps:
	rustup add component clippy
	rustup add component fmt

.PHONY: test
test:
	cargo test
	cargo clippy

.PHONY: doc
doc: docs
.PHONY: docs
docs:
	cargo doc -j4 -q
