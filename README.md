# rust-vulkan

I'm just following the guide at https://kylemayes.github.io/vulkanalia

## Progress

Finished chapter [33. Generating mipmaps](https://kylemayes.github.io/vulkanalia/quality/generating_mipmaps.html)

## Prerequisites

You need VulkanSDK and rust installed in order to build this project.

You also need git and podman (container tool) to run the ´compile_shaders.sh´ script. 
You can modify this step to use docker or compile the shaders natively if you like. 

## Instructions

1. Build shaders
    ```
    $ ./compile_shaders.sh
    ```
2. Build application
    ```
    $ cargo build
    ```
3. Run application
    ```
    $ cargo run
    ```