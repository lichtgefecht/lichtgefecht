all:
	rm -f sdkconfig
	rm -f sdkconfig.defaults 
	cat sdkconfig.platform sdkconfig.secret > sdkconfig.defaults
	idf.py -p /dev/ttyACM0 flash monitor

clean:
	rm -f sdkconfig
	rm -f sdkconfig.defaults
	rm -rf build