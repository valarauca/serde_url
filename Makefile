
.PHONY: install-deps
install-deps:
	rustup add component clippy
	rustup add component fmt

.PHONY: test
test:
	cargo test
	cargo clippy
	cargo fmt -- -f
	rm -rf src/*.rs.bk

.PHONY: doc
doc: docs
.PHONY: docs
docs:
	cargo doc -j4 -q
	cp docs/index.html target/doc/index.html
	rm -rf docs
	mv target/doc docs
