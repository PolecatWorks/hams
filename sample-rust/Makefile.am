
CARGO_TARGET =$(if $(RELEASE),target/release,target/debug)
CARGO_BUILD_FLAGS =$(if $(RELEASE),--release,)

IMAGE_NAME=polecatworks/sample-rust
IMAGE_TAG=0.1.0

REGISTRY_URL=dockerreg.k8s:5000


$(top_build_prefix)$(CARGO_TARGET)/sample-rust:
	@$(CARGO) build $(CARGO_BUILD_FLAGS)


check-local: $(top_build_prefix)$(CARGO_TARGET)/sample-rust
	@$(top_build_prefix)$(CARGO_TARGET)/sample-rust start

clean-local:
	@$(CARGO) clean $(CARGO_BUILD_FLAGS)

docker-build:
	@docker build .. -t $(IMAGE_NAME) -f Dockerfile
	@docker image ls $(IMAGE_NAME)

docker-run: docker-build
	@docker run -it $(IMAGE_NAME)

docker-push: docker-build
	@docker tag $(IMAGE_NAME) $(REGISTRY_URL)/$(IMAGE_NAME):$(IMAGE_TAG)
	@docker push $(REGISTRY_URL)/$(IMAGE_NAME):$(IMAGE_TAG)
