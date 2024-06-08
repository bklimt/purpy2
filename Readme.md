
# Purpy

Purpy is a 2D platformer game engine, originally developed in python using pygame+opengl and then ported to rust. It has multiple renderers, depending on either SDL2, WGPU, Winit, or some combination of the above.

For information about how to develop levels for Purpy, see [purpy for python](https://github.com/bklimt/purpy). The default game assets are also in that repo.

## purpy

The default library provides multiple backends, depending on which optional dependencies are provided:

* `sdl2` - SDL can provide multiple features:
  * `SdlRenderer` - A renderer which uses SDL textures for basic 2D rendering. This lacks some of the render post-processing provided by the alternative WGPU renderer.
  * `SdlSoundManager` - Allows purpy to play WAVE sound files using SDL2.
  * Methods on `InputManager` for processing SDL key events.
  * Glue for using SDL windows with the WGPU renderer, if WGPU is also included.
* `winit` - Provides glue for using Winit instead of SDL for the main game loop. When using Winit, you must use WGPU for your renderer. This dependency provides:
  * Methods on `InputManager1 for processing winit key events.
  * Glue for using winit `Window`s with the WGPU renderer, assuming WGPU is also included.
* `wgpu` - A WebGPU-based rendering engine, which is faster than the SDL renderer, with prettier effects. Provides the following:
  * `WgpuRenderer` - The renderer based on WGPU, which can be used with either SDL or Winit windows.

For a standalone game executable, it is recommended to use `sdl2`+`wgpu`. For a WASM web game, it is recommended to use `winit`+`wgpu`.

## Texture Atlas

The current version of purpy requires all textures to be packed into a single file, with an index for which file ended up where in the combined image. First, make sure all assets are present in a directory called `assets`. To generate the texture atlas, use the included tool:
```
cargo run --bin create_texture_atlas -- \
  --texture-list ../purpy/assets/textures.txt \
  --texture-atlas-image ../purpy/assets/textures2.png \
  --texture-atlas-index ../purpy/assets/textures_index2.txt \
  --score-func multiply \
  --try-all-pairs
```

## Dependencies

To install dependencies in debian linux:

```
sudo apt install libudev-dev libsdl2-dev libsdl2-image-dev
```

## purpy_sdl

This is a standalone implementation of the purpy game using only SDL for windowing and rendering.

```
cargo run --bin purpy_sdl
```

## purpy_wgpu

This is a standalone implementation of the purpy game using SDL for windowing and WGPU for rendering.

```
cargo run --bin purpy_wgpu
```

## purpy_winit

This is a standalone implementation of the purpy game using Winit for windowing and WGPU for rendering.

```
cargo run --bin purpy_winit
```

## purpy_wasm

This is an implementation of the purpy game for use as a WASM web app.

To build purpy for WASM, you need to have `wasm-pack` installed:
```
cargo install wasm-pack
```

Before building purpy for WASM, you have to package the various resources into a .tar.gz archive, which will be included in the build. It must be placed in the root of the repo. Build it using the built-in tool:
```
cargo run --bin package_assets -- \
  --manifest=assets/manifest.txt \
  --output assets.tar.gz
```

Build purpy for WASM:

```
wasm-pack build purpy_wasm --target web
```

Run a testing server with purpy:
```
cd purpy_wasm
python3 -m http.server
```

To update the hosted version (from the repo root):
```
rm -r docs/purpy_wasm
cp purpy_wasm/pkg docs/purpy_wasm
```

