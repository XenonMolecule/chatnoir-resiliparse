.PHONY: profile clean

profile:
	dotnet publish -c Release -o publish
	printf '%s\n' '#!/usr/bin/env sh' 'DIR=$$(CDPATH= cd -- "$$(dirname -- "$$0")" && pwd)' 'exec dotnet "$$DIR/publish/profile.dll" "$$@"' > profile
	chmod +x profile

clean:
	dotnet clean
	rm -rf bin obj publish profile
