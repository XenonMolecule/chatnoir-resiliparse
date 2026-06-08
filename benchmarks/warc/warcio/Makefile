.PHONY: profile clean

profile:
	python3 -m venv venv
	. venv/bin/activate
	venv/bin/python3 -m pip install -r requirements.txt
	printf '%s\n' '#!/usr/bin/env sh' '. venv/bin/activate' 'python3 profile.py "$$@"' > profile
	chmod +x profile

clean:
	rm -rf venv
	rm -f profile
