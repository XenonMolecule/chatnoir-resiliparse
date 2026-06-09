.PHONY: profile clean

profile:
	mvn -q package
	printf '%s\n' '#!/usr/bin/env sh' 'exec java -cp "target/classes:target/dependency/*" --enable-native-access=ALL-UNNAMED Profile "$$@"' > profile
	chmod +x profile

clean:
	mvn -q clean
	rm -f profile
