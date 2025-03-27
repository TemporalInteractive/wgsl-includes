#[cfg(test)]
mod tests {
    use wgsl_includes::include_wgsl;

    #[test]
    fn include_wgsl() {
        let shader_src = include_wgsl!("tests/shader.wgsl");
        println!("{}", shader_src);
    }

    #[test]
    fn include_wgsl_resolve() {
        let shader_src = include_wgsl!("tests/shader_resolve.wgsl");
        println!("{}", shader_src);
    }
}
