.PHONY: reflector tagger

all: reflector tagger

reflector: proto-c
	cd components/codec && cmake -S . -B build && cmake --build build --parallel 24 && ./build/codecTest
	cd reflector && cargo make

tagger: proto-c 
	make -C tagger

proto-c:
	protoc-c --c_out=components/codec/api --proto_path=api what.proto