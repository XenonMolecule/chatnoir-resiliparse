.PHONY: profile clean

profile:
	cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
	cmake --build build --config Release
	cp build/profile profile

clean:
	rm -rf build profile
