# Penare

A wonky distortion plugin :3 

## Features
- Pre and Post gain control (no way)
- Filter control (like which range the distortion is applied)
- Waveshaper :O (I think that's what it's called)
  | Function Type     | Notation
  |-------------------|----------
  | Classic Hard Clip | $max(min(x, A), -A)$
  | Scaled Clip       | $x * A$
  | 2Tanh             | $tanh(2x) * A$
  | Repiprocal        | $2 * sign(x) * (A - \frac{A}{\|x\| + 1})$
  | Softdrive         | $sign(x) * R \text{ where } R = \begin{cases} 2x & \text{ for } 0 \leq \|x\| \leq \frac{A}{3} \\ \frac{3 - (2 - 3x)^2}{3} & \text{ for } \|x\| \leq \frac{A}{3} \\ A & \text{ otherwise } \end{cases}$
  | Inv2Tanh          | $sign(x) * R \text{ where } R = \begin{cases} A * tanh(2atanh(\frac{2\|x\|}{A})) & \text{for } x \lt \frac{A}{2} \\ A & \text{otherwise}\end{cases}$

  where $x$ is the input sample and $A$ is the value of the `Function Parameter` slider
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
