.PHONY: publish

publish_api:
	cargo release $(bump) --execute
	git push gitlab && git push gitlab --tags


# CLI
install:
	cargo install --path crates/cli
gitlab_publish_cli:
	cargo release $(bump) --execute
	git push gitlab && git push gitlab --tags
github_publish_cli:
	cargo release $(bump) --execute
	git push github && git push github --tags
