RELEASE ?=

NAME=hams

SUBDIRS = sample-c sample-ts sample-rust sample-kotlin

CARGO_TARGET = $(if $(RELEASE), target/release, target/debug)
CARGO_BUILD_FLAGS = $(if $(RELEASE), --release,)

ACLOCAL_AMFLAGS = -I m4



$(CARGO_TARGET)/libhams.dylib $(top_build_prefix)$(CARGO_TARGET)/sample-rust:
	$(CARGO) build $(CARGO_BUILD_FLAGS)

status:
	@echo Cargo build: $(if $(RELEASE), release, debug)
	@echo Cargo flags: $(CARGO_BUILD_FLAGS)
	@echo Cargo target: $(CARGO_TARGET)

clean-local:
	cd $(srcdir)/$(RSHELLO_DIR); cargo clean


DOCKER=docker

.ONESHELL:
docker-build:
	{ \
	$(DOCKER) build . -t $(NAME) -f Dockerfile; \
	$(DOCKER) image ls $(NAME); \
	}

docker:
	$(DOCKER)  build . -t $(NAME) -f Dockerfile

dockerx:
	$(DOCKER)  buildx build . -t $(NAME) -f Dockerfile --platform linux/arm64/v8,linux/amd64




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


cargo-miri-watch:
	@cargo watch -x 'miri test'

cargo-bloat:
	@cargo bloat --release -n 10

doc: cargo-doc

cargo-doc:
	@cargo doc --no-deps --document-private-items

cargo-fmt:
	@cargo fmt --all -- --check

cargo-clippy:
	@cargo clippy

cargo-coverage:
	@cargo tarpaulin --ignore-tests --out Lcov

cargo-criterion:
	@cargo criterion
	@open target/criterion/report/index.html

git-mergedbranch-delete:
	@git branch --merged | egrep -v "(^\*|master|main|dev)" | xargs git branch -d
