#!/bin/bash

git submodule update --init --recursive

podman build --rm -t shaderc/shaderc -f shaderc/Dockerfile shaderc

echo "permission test"

echo "before"
podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code:Z shaderc/shaderc ls -la
podman run --rm -v $PWD/assets/shaders:/code:Z shaderc/shaderc ls -la

podman unshare chown $(id -u):$(id -g) -R $PWD/assets/shaders

echo "after"
podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code:Z shaderc/shaderc ls -la
podman run --rm -v $PWD/assets/shaders:/code:Z shaderc/shaderc ls -la

podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code:Z shaderc/shaderc glslc shader.frag -o frag.spv
podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code:Z shaderc/shaderc glslc shader.vert -o vert.spv
