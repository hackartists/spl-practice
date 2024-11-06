PROGRAM_ID ?= $(shell cat PROGRAM_ID)
BUFFER_KEY ?= 5vWmH4dKgCpqrSQDNtMhR1HDHcrJMuMLbkAMgzCX9JKU
ADDRESS ?= $(shell solana address)
VERSION ?= 1
PAYER_KEYPAIR_FILE ?= ~/.config/solana/id.json

BUILD_ENV ?= PROGRAM_ID=$(PROGRAM_ID) BUFFER_KEY=$(BUFFER_KEY) ADDRESS=$(ADDRESS) VERSION=$(VERSION) PAYER_KEYPAIR_FILE=$(PAYER_KEYPAIR_FILE)

.PHONY: run
run:
	$(BUILD_ENV) cargo run -p app

.PHONY: build
build:
	cargo build-sbf --manifest-path=program/Cargo.toml --sbf-out-dir=dist/program

deploy: build
	mkdir -p build/program-deploy-$(VERSION)
	solana program deploy dist/program/expiry_token.so --keypair ~/.config/solana/id.json > build/program-deploy-$(VERSION)/deploy.log
	cat build/program-deploy-$(VERSION)/deploy.log | grep "Program Id" | awk '{print $$3}' > PROGRAM_ID
	cat PROGRAM_ID

write-buffer:
	solana program write-buffer dist/program/expiry_token.so

build/program-%: build
	mkdir -p $@
	solana program write-buffer dist/program/expiry_token.so > $@/write-buffer.log
	cat $@/write-buffer.log | grep "Buffer" | awk '{print $$2}' > $@/program-id

upgrade: build/program-$(VERSION)
	$(eval BUFFER_KEY=$(shell cat build/program-$(VERSION)/program-id))
	solana program upgrade $(BUFFER_KEY) $(PROGRAM_ID) --keypair ~/.config/solana/id.json

clean:
	rm -rf dist build PROGRAM_ID

airdrop:
	solana airdrop 5 $(ADDRESS)
