all: build

build:
	rm -f sdkconfig
	rm -f sdkconfig.defaults 
	cat sdkconfig.platform sdkconfig.secret > sdkconfig.defaults
	idf.py --ccache build

flash: build
	idf.py -p /dev/ttyACM0 --ccache flash monitor

flash2: build
	idf.py -p /dev/ttyACM1 --ccache flash monitor

clean:
	rm -f sdkconfig
	rm -f sdkconfig.defaults
	rm -rf build
