#!/bin/bash

git submodule update --recursive

docker build --rm -t shaderc/shaderc -f shaderc/Dockerfile shaderc

docker run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code shaderc/shaderc glslc shader.frag -o frag.spv
docker run --rm -u $(id -u):$(id -g) -v $PWD/assets/shaders:/code shaderc/shaderc glslc shader.vert -o vert.spv
