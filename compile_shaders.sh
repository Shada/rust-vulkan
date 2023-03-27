#!/bin/bash

git submodule update --init --recursive

podman pull shadahub/shaderc || podman build --rm -t shadahub/shaderc -f shaderc/Dockerfile shaderc

podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code:Z shadahub/shaderc glslc shader.frag -o frag.spv
podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code:Z shadahub/shaderc glslc shader.vert -o vert.spv
