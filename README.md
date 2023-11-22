
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
- ~~Add visibility graph for chunk culling~~
- ~~Compress data to send to GPU for each chunk.~~

- ~~Calculate frustrum corners and cull based on that.~~
- ~~Fix visibility culling (If we've already seen a chunk, we might want to check if we can get through from another side than the one already considered).~~
- ~~Load chunks around player first.~~
- Add lighting (+ dithering between light levels).
- Add fog.

- Add loading screen
- Make sure build works on WASM target
- Be able to add/remove blocks

- Load game data from JSON (or other file format) for example block models, etc...

Minecraft tends to render 100-200 chunks per frame. On my computer, at 6 render distance, it gets a variable 60fps. This should be achievable here.
// calculate frustum bounds and use to cull.
// change rendering code so we are not rebinding the pipeline for every single chunk.