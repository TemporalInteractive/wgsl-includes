# 🏗️ wgsl-includes

[![wgsl-includes crate](https://img.shields.io/crates/v/wgsl-includes.svg)](https://crates.io/crates/wgsl-includes)

This tiny crate is meant to solve two major pain points with naga & wgsl: no validation at compile time and no support for shader includes. Note that shaders are only included ONCE, meaning circular includes are not allowed. All shader paths must be relative to the crate root directory.

This crate is nightly only.

## Features
- Wgsl shader includes
- Shader syntax validation

## Example
`my_crate/shaders/shared.wgsl` contents:
```rust
// Define shared logic in a separate wgsl shader, this file is free to include more files
const VALUE: u32 = 0;
```

`my_crate/shaders/some_compute_shader.wgsl` contents:
```rust
// We can now include our shared logic
@include shaders/shared.wgsl

@compute
@workgroup_size(128)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // And use the contents from `shared.wgsl`
    let maths: u32 = global_id.x * VALUE;
}
```

On the rust side we can now include and validate our shader source code as follows:
```rust
let shader_src: &str = include_wgsl!("shaders/some_compute_shader.wgsl");
```