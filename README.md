# Penare

A wonky distortion plugin :3 

## Features
- Pre and Post gain control (no way)
- Filter control (like which range the distortion is applied)
- [Waveshaper](https://github.com/azur1s/penare/wiki/Waveshapers) :O (I think that's what it's called)
- Rectify
  - Half and Full rectify
  - Mix between dry and wet signal
  - Mix in wet signal (in case you want to layer it)

## Contributing
Some of the [algorithms](src/clip.rs) probably is not correct so you can fix it if you think it's incorrect. Or you can add new one if you're feeling generous :D

## Building

The `build.ps1` and `build_debug.ps1` is for me to use. Although you can use it, if it works.

Use this to compile to VST3 and CLAP.

```shell
cargo xtask bundle penare --release
```
