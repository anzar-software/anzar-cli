.PHONY: publish

install:
	cargo install --path crates/cli

publish:
	cargo release $(bump) --package anzar-shared --package anzar-cli --execute
	git push github && git push github --tags

test:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	DB=sqlite cargo test
	docker run -d --name mongodb \
	 -e MONGO_INITDB_ROOT_USERNAME=hakou \
	 -e MONGO_INITDB_ROOT_PASSWORD=password \
	 -e MONGO_INITDB_DATABASE=dev \
	 -p 27017:27017 \
	 mongo:7.0
	DB=mongodb cargo test
	docker stop mongodb && docker rm mongodb
	echo "Mongodb successed\n"
	docker run -d --name postgresql \
  -e POSTGRES_USER=hakou \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=dev \
  -p 5432:5432 \
  postgres:16-alpine
	DB=postgresql cargo test
	echo "Postgrs successed\n"
	docker stop postgresql && docker rm postgresql
	docker build -t hakouguelfen79/anzar:latest .

