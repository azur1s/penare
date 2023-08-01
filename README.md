# Penare

A semi-soft-clipper and/or semi-distortion plugin :3

You probably would not want to use this as a mixing tool
(as the algorithm used might not be correct), rather you could
use it as an experimental distortion kind-of plugin

## Features
- Mix wet and dry signal!?
- Pre and Post gain control
- Filter control (like which range the clipping is applied)
- Multiple clipping types!!
  - Hard
  - 2Tanh (Tanh but multiplied by 2)
  - Repiprocal

## Contributing
Some of the [algorithms](src/clip.rs) probably is not correct so you can fix it if you think it's incorrect. Or you can add new one if you're feeling generous :D

## Building

The `build.ps1` and `build_debug.ps1` is for me to use. Although you can use it, if it works.

Use this to compile to VST3 and CLAP.

```shell
cargo xtask bundle penare --release
```
