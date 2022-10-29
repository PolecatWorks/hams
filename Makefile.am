RELEASE ?=

SUBDIRS = src-c

CARGO_TARGET = $(if $(RELEASE), target/release, target/debug)
CARGO_BUILD_FLAGS = $(if $(RELEASE), --release,)

ACLOCAL_AMFLAGS = -I m4


$(CARGO_TARGET)/libhams.dylib:
	$(CARGO) build $(CARGO_BUILD_FLAGS)

status:
	@echo Cargo build: $(if $(RELEASE), release, debug)
	@echo Cargo flags: $(CARGO_BUILD_FLAGS)
	@echo Cargo target: $(CARGO_TARGET)

clean-local:
	cd $(srcdir)/$(RSHELLO_DIR); cargo clean

cargo-watch:
	@cargo watch -x 'build'

cargo-build:
	@cargo build $(CARGO_BUILD_FLAGS)

cargo-release:
	@cargo build --release

cargo-version:
	@cargo --version

cargo-test:
	@cargo test -- --nocapture

cargo-test-watch:
	@cargo watch -x 'test -- --nocapture'

cargo-bloat:
	@cargo bloat --release -n 10

cargo-doc:
	@cargo doc --no-deps --open

cargo-fmt:
	@cargo fmt --all -- --check

cargo-clippy:
	@cargo clippy

cargo-coverage:
	@cargo tarpaulin --ignore-tests --out Lcov

cargo-criterion:
	@cargo criterion
	@open target/criterion/report/index.html
