.PHONY: reflector tagger

all: reflector tagger

reflector:
	cd reflector && cargo make

tagger:
	make -C tagger