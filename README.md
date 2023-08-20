# Asteroids
Simple desktop Asteroids game

To setup a rust project:
https://doc.rust-lang.org/edition-guide/editions/creating-a-new-project.html

Windowing library:
https://crates.io/crates/winit

To use shaders we need a compiler, which can be found here:
https://github.com/microsoft/DirectXShaderCompiler/releases

More info on the compiler and shader language:
https://github.com/microsoft/DirectXShaderCompiler/wiki

An instance is the first thing we create when using wgpu, and is primarily used to initialize a connection to a graphics device. An adapter is the handle to the actual graphics card we obtain from the instance. A surface is the section of screen that we will draw to. The only requirement here is that it needs to implement the HasRawWindowHandle trait, which winit does.

cargo run --features bevy/dynamic_linking