PROGRAM_ID ?= $(shell cat PROGRAM_ID)
BUFFER_KEY ?= 5vWmH4dKgCpqrSQDNtMhR1HDHcrJMuMLbkAMgzCX9JKU
VERSION ?= 1

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
