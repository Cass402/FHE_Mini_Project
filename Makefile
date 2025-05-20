.PHONY: all build test run run-interactive docker docker-interactive clean

all: build

build:
	cargo build --release

test:
	cargo test --release

run:
	cargo run --release

run-interactive:
	cargo run --release --example interactive_demo

docker:
	docker-compose build
	docker-compose run fhe-demo

docker-interactive:
	docker-compose build
	docker-compose run interactive-demo

clean:
	cargo clean
	rm -rf outputs/*.png