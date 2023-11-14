
# mcrs
This is a minecraft-clone written in rust. It is made with WGPU and is playable on the web.

## Upcoming features
- Player physics
- ~~Better meshing algorithm~~
- Interesting generation
- LOD meshing
- Saving/loading chunks

- Maybe try out libloading for loading plugins (idt WASM will cut it, there's no passing mutable data)

## Steps to finish
- ~~Document engine classes~~
- ~~Make better player jump/movement physics~~
- ~~Add ImGui to configure variables~~
- ~~Make player physics actually work better~~
- Add visibility graph for chunk culling
- Add lighting (+ dithering between light levels)
- Compress data to send to GPU for each chunk.

- Add loading screen
- Make sure build works on WASM target
- Be able to add/remove blocks

- Load chunks around player first
- Sort rendering code more
- Load game data from JSON (or other file format) for example block models, etc...
- Run flamegraph and see why minimal FPS
- Add structure generation
    - Create a chunk loader for separate stages of generation
- Frustum culling
- Create visibility graph
- Cut down on memory usage - pack bits