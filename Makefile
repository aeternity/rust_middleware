# rust build
GIT_DESCR = $(shell git describe --always)
PROJECT_NAME = aepp-middleware
# build output folder
OUTPUTFOLDER = target
# docker image
DOCKER_REGISTRY = aeternity
DOCKER_IMAGE = aepp-middleware
DOCKER_TAG = $(shell git describe --always)

default: build

build: build-dist

build-dist:
	@echo rust build
	cargo build
	@echo done

clean:
	@echo remove $(OUTPUTFOLDER) folder
	@rm -rf $(OUTPUTFOLDER)
	@echo done

docker-build:
	@echo build image
	docker build -t $(DOCKER_IMAGE) -f ./docker/Dockerfile .
	@echo done

docker-push:
	@echo push image
	docker tag $(DOCKER_IMAGE) $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):$(DOCKER_TAG)
	docker push $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):$(DOCKER_TAG)
	@echo done

