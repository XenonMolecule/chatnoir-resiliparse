.PHONY: profile clean

profile:
	npm install
	printf '%s\n' '#!/usr/bin/env sh' 'exec node "$$(dirname "$$0")/profile.js" "$$@"' > profile
	chmod +x profile

clean:
	rm -rf node_modules package-lock.json profile
