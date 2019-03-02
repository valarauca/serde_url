
.PHONY: test
test:
	cargo test

.PHONY: doc
doc: docs
.PHONY: docs
docs:
	cargo doc -j4 -q
