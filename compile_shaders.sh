#!/bin/bash

git submodule update --init --recursive

podman build --rm -t shaderc/shaderc -f shaderc/Dockerfile shaderc

podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code shaderc/shaderc glslc shader.frag -o frag.spv
podman run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code shaderc/shaderc glslc shader.vert -o vert.spv
