# Penare

A wonky distortion plugin :3 

## Features
- Pre and Post gain control (no way)
- Filter control (like which range the distortion is applied)
- Symmetric and Asymmetric waveshaping! ([Waveshapers list](https://github.com/azur1s/penare/wiki/Waveshapers))
- Rectifier
  - Half and Full rectify
  - Mix between dry and wet signal
  - Mix in wet signal (in case you want to layer it)
- Floorer
  - Adjustable step (you can use this for quality reduction kind-of deal)
  - Mix between or in dry and wet signal

## Contributing
Some of the algorithms probably is not correct so you can fix it if you think it's incorrect. Or you can add new one if you're feeling generous :D

## Building

The `build.ps1` and `build_debug.ps1` is for me to use. Although you can use it, if it works.

Use this to compile to VST3 and CLAP.

```shell
cargo xtask bundle penare --release
```
