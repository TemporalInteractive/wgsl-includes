#[cfg(test)]
mod tests {
    use wgsl_includes::include_wgsl;

    #[test]
    fn include_wgsl() {
        let shader_src = include_wgsl!("shaders/shader.wgsl");
        println!("{}", shader_src);
    }

    #[test]
    fn include_wgsl_resolve() {
        let shader_src = include_wgsl!("shaders/shader_resolve.wgsl");
        println!("{}", shader_src);
    }

    #[test]
    fn include_wgsl_mixed_line_endings() {
        let shader_src = include_wgsl!("shaders/mixed_line_endings.wgsl");
        assert!(shader_src.contains("const VALUE") && shader_src.contains("const OTHER"));
    }

    #[test]
    fn include_wgsl_at_end_of_file() {
        let shader_src = include_wgsl!("shaders/include_at_eof.wgsl");
        assert!(shader_src.contains("const VALUE"));
    }
}
