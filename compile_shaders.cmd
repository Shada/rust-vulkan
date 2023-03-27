git submodule update --init --recursive

podman build --rm -t shaderc/shaderc -f shaderc/Dockerfile shaderc

podman run --rm -v ./assets/shaders:/code shaderc/shaderc glslc shader.frag -o frag.spv
podman run --rm -v ./assets/shaders:/code shaderc/shaderc glslc shader.vert -o vert.spv
