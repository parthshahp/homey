# justfile

set dotenv-load := true

# ====== Config ======

image := "ghcr.io/parthshahp/homey"
tag := "latest"
platforms := "linux/amd64,linux/arm64"

# ====================

default:
    @just --list

# One-time: ensure buildx builder exists and can do multi-arch
setup:
    docker buildx create --name multiarch --use || true
    docker buildx inspect --bootstrap

# Build + push multi-arch image
publish: setup
    docker buildx build --platform {{ platforms }} -t {{ image }}:{{ tag }} --push .

# Convenience: build locally for quick testing (native arch only)
build-local:
    docker build -t {{ image }}:dev .

run-local: build-local
    docker run --rm -p 3000:3000 {{ image }}:dev
