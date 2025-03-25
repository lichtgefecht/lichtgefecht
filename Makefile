.PHONY: reflector tagger

all: reflector tagger

reflector: proto-c
	cd reflector && cargo test && cargo build

tagger: proto-c 
	make -C tagger

flash-tagger: tagger
	make -C tagger flash

flash-tagger2: tagger
	make -C tagger flash2
	
proto-c:
	protoc-c --c_out=components/lg_api --proto_path=api lg.proto

clean:
	rm -rf components/codec/build
	rm -rf components/diag/build
	rm -rf components/peripherals/build
	make -C tagger clean

clean-all: clean
	cd reflector && cargo clean
