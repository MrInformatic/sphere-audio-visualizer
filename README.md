# Sphere Audio Visualizer

The Sphere Audio Visualizer is a cross-platform audio visualizer based on GPU-accelerated visualizations. The shader used is implemented in both implemented in rust using [rust-gpu](https://github.com/EmbarkStudios/rust-gpu) and [WGSL](https://gpuweb.github.io/gpuweb/wgsl/). Thanks to [WGPU](https://wgpu.rs/) the software is compatible with Vulkan, D3D12, D3D11, Metal, WebGPU, WebGL and OpenGLES. Thanks to [GStreamer](https://gstreamer.freedesktop.org/) the software can export the results to a variety of encodings.

## Table of content

- [Build from source](#build-from-source)
- [Author](#author)
- [License](#license)
- [Acknowledgments](#acknowledgments)

## Build from source

### Prerequisites

Please go to [rustup.rs](https://rustup.rs/) and follow the 
instructions. It will be quick and painless (hopefully).

### Build

If you have installed rustup and successfully cloned the 
repository, you can go ahead and open a terminal in the project
folder and run: 

```
cargo run --release
```

If you have a problem running the appropriate Command on the operating system 
of your choice fears not opening an issue. 
I do not have all the operating systems at my disposal to test
all the backends.

## Author

* **Philipp Haustein** - [MrInformatic](https://github.com/MrInformatic)

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details

## Acknowledgments

- WGPU <https://wgpu.rs/>
- WInit <https://github.com/rust-windowing/winit>
- rust-gpu <https://github.com/EmbarkStudios/rust-gpu>
- WGSL <https://gpuweb.github.io/gpuweb/wgsl/>
- EGUI <https://github.com/emilk/egui>
- Rapier <https://www.rapier.rs/>
- Rayon <https://github.com/rayon-rs/rayon>
- GStreamer <https://gstreamer.freedesktop.org/>
