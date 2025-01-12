all:
	rm -f sdkconfig
	rm -f sdkconfig.defaults 
	cat sdkconfig.platform sdkconfig.secret > sdkconfig.defaults
	idf.py --ccache build

clean:
	rm -f sdkconfig
	rm -f sdkconfig.defaults
	rm -rf build