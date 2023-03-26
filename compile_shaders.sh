#!/bin/bash

echo "$(id -u):$(id -g)"

ls -la

git submodule update --init --recursive

podman build --rm -t shaderc/shaderc -f shaderc/Dockerfile shaderc

podman run --rm -u $(id -u):$(id -g) -v ./assets/shaders:/code shaderc/shaderc ls -la
podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code shaderc/shaderc ls -la
podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code shaderc/shaderc echo "$(id -u):$(id -g)"
podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code:rw shaderc/shaderc glslc shader.frag -o frag.spv
podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code:rw shaderc/shaderc glslc shader.vert -o vert.spv
