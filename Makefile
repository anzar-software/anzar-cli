.PHONY: publish

install:
	cargo install --path crates/cli

publish:
	cargo release $(bump) --package shared --package cli --execute
	git push github && git push github --tags
