#!/bin/bash

git submodule update --init --recursive

docker pull docker.io/shadahub/shaderc || docker build --rm -t shadahub/shaderc -f shaderc/Dockerfile shaderc

docker run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code shadahub/shaderc glslc shader.frag -o frag.spv
docker run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code shadahub/shaderc glslc shader.vert -o vert.spv
