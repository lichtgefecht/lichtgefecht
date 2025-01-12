.PHONY: reflector tagger

all: reflector tagger

reflector:
	cd reflector && cargo make

tagger: proto-c 
	make -C tagger

proto-c:
	protoc-c --c_out=components/codec/api --proto_path=api what.proto