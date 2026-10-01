.PHONY: server client publish dev

server:
	cd server && spacetime build

publish:
	spacetime publish --server local -p ./server the-bastion

client:
	cargo run -p client

dev: publish client
