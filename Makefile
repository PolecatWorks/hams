IMAGE_NAME=rusty-microservice/rusty
REGISTRY=dockerreg.k8s:5000


build:
	cargo build

build-watch:
	@cargo watch -x 'build'

release:
	cargo build --release

status:
	@cargo --version

test:
	@cargo test -- --nocapture

test-watch:
	@cargo watch -x 'test -- --nocapture'

run:
	@cargo run -- listen

docker-build:
	@docker build -t $(IMAGE_NAME) .

docker-publish: docker-build
	@docker tag ${IMAGE_NAME} ${REGISTRY}/${IMAGE_NAME}
	@docker push ${REGISTRY}/${IMAGE_NAME}


docker-shell: docker-build
	@docker run -it --entrypoint /bin/bash $(IMAGE_NAME)

docker-tag: docker
	@docker tag rust_hello:latest rust_hello:1.0.0

rollout:
	@kubectl rollout restart deployment rusty

bloat:
	@cargo bloat --release -n 10

k8sshell:
	@kubectl run -i --tty alpine --image=alpine:latest --rm --restart=Never -- sh -c "apk add curl  && exec sh"

doc:
	@cargo doc --no-deps --open
doc-watch:
	@cargo watch -x 'doc --no-deps --workspace --open'

style-check:
	@cargo fmt --all -- --check

lint:
	@cargo clippy

benchmark:
	@cargo criterion
	@open target/criterion/reports/index.html
