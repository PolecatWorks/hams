
CARGO_TARGET =$(if $(RELEASE),target/release,target/debug)
CARGO_BUILD_FLAGS =$(if $(RELEASE),--release,)

$(top_build_prefix)$(CARGO_TARGET)/sample-rust:
	@$(CARGO) build $(CARGO_BUILD_FLAGS)


check-local: $(top_build_prefix)$(CARGO_TARGET)/sample-rust
	@$(top_build_prefix)$(CARGO_TARGET)/sample-rust start
