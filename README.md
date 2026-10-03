# Vulkan Triangle in Rust using Vulkanalia

This repo has two siblings
- [Vulkan Triangle in Rust using Ash](https://github.com/barlog-m/vulkan-triangle-rust-ash)
- [Vulkan Triangle in C](https://github.com/barlog-m/vulkan-triangle-c)

This is an example that draws a rectangle in Vulkan.

It draws a rectangle instead of a triangle because it uses shaders from the official [Vulkan
example](https://docs.vulkan.org/tutorial/latest/04_Vertex_buffers/03_Index_buffer.html).

The example targets Vulkan 1.4, uses dynamic rendering, synchronization 2, unified image layout, and enables a bunch of
unnecessary stuff just because why not.

The example uses [Vulkan Memory Allocator](https://github.com/GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator), because nobody wants to allocate memory manually.

It also separates the code into modules and functions instead of providing you with a one-billion-lines single file.

The example uses [Vulkanalia](https://github.com/KyleMayes/vulkanalia)

AI was used to make this code exists, but good luck to write same code only with AI.

To make this example work you have to compile that lonely shader by executing `compile.sh` or `compile.ps1` inside `shaders` folder.

You also need to have the [Vulkan SDK](https://www.vulkan.org) and [Rust](https://rust-lang.org) installed.
