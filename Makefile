setup:
	git config core.hooksPath .githooks
	cargo install cargo-modules --no-default-features

check:
	./.githooks/pre-commit
