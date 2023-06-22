# rust-vulkan

I'm just following the guide at https://kylemayes.github.io/vulkanalia

## Progress

Finished chapter [37. Secondary command buffers](https://kylemayes.github.io/vulkanalia/dynamic/secondary_command_buffers.html)

## Prerequisites

You need VulkanSDK and rust installed in order to build this project.

You also need git and docker (container tool) to run the `compile_shaders.sh` script.
You can modify this step to compile the shaders natively if you like.

## Instructions

1. Build shaders

    ```console
    ./compile_shaders.sh
    ```

2. Build application

    ```console
    cargo build
    ```

3. Run application

    ```console
    cargo run
    ```
