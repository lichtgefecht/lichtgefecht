.PHONY: reflector tagger

all: reflector tagger

reflector: proto-c
	cd components/codec && cmake -S . -B build -DTESTING=ON && cmake --build build --parallel 24 && ./build/codecTest
	cd reflector && cargo test && cargo build

tagger: proto-c 
	make -C tagger

flash-tagger: tagger
	make -C tagger flash
	
proto-c:
	mkdir -p components/codec/build/api
	protoc-c --c_out=components/codec/build/api --proto_path=api lg.proto

clean:
	rm -rf components/codec/build
	rm -rf components/diag/build
	rm -rf components/peripherals/build
	make -C tagger clean

clean-all: clean
	cd reflector && cargo clean