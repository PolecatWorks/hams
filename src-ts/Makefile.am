
CARGO_TARGET =$(if $(RELEASE),target/release,target/debug)
CARGO_BUILD_FLAGS =$(if $(RELEASE),--release,)

$(top_build_prefix)$(CARGO_TARGET)/libhams.dylib:
	cd $(top_srcdir) && make $(CARGO_TARGET)/libhams.dylib

libhams.dylib: $(top_build_prefix)$(CARGO_TARGET)/libhams.dylib
	cp $(top_build_prefix)$(CARGO_TARGET)/libhams.dylib ./

node_modules package-lock.json: package.json
	@npm i
	@touch package-lock.json node_modules

dist/cli.js: src/cli.ts src/commands/*
dist/cli.js: | node_modules
	@npm run build

clean-local:
	@rm -rf dist
	@rm -rf node_modules

check-local: libhams.dylib package-lock.json | node_modules
	@npm run ts-start start
