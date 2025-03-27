use std::path::{Path, PathBuf};

use naga::{
    front::wgsl,
    valid::{Capabilities, ValidationFlags, Validator},
};
use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

fn resolve_shader_path<T: Into<PathBuf>>(shader_path: T) -> PathBuf {
    let dir = std::env::current_dir().unwrap();
    dir.join(shader_path.into())
}

fn parse_shader_includes_recursive(name: &str, includes: &mut Vec<String>) -> String {
    let file_path = Path::new(&resolve_shader_path(name))
        .to_str()
        .unwrap()
        .to_owned();

    if includes.contains(&file_path) {
        return String::new();
    }
    includes.push(file_path.clone());

    let mut contents = std::fs::read_to_string(&file_path).unwrap_or_else(|_| {
        panic!(
            "Failed to include shader \"{}\".",
            file_path.replace("\\", "/")
        )
    });

    let mut include_indices: Vec<usize> = contents.match_indices("@include").map(|i| i.0).collect();
    include_indices.reverse();
    for include_index in include_indices {
        let end_of_line = contents[include_index..].find('\n').unwrap() + include_index - 1;
        let include_name = contents[(include_index + 9)..end_of_line].to_owned();

        for i in (include_index..end_of_line).rev() {
            contents.remove(i);
        }
        contents.insert_str(
            include_index,
            &parse_shader_includes_recursive(&include_name, includes),
        );
    }

    contents
}

fn parse_shader_includes(mut contents: String) -> String {
    let mut includes = vec![];

    let mut include_indices: Vec<usize> = contents.match_indices("@include").map(|i| i.0).collect();
    include_indices.reverse();
    for include_index in include_indices {
        let end_of_line = contents[include_index..].find('\n').unwrap() + include_index - 1;
        let include_name = contents[(include_index + 9)..end_of_line].to_owned();

        for i in (include_index..end_of_line).rev() {
            contents.remove(i);
        }
        contents.insert_str(
            include_index,
            &parse_shader_includes_recursive(&include_name, &mut includes),
        );
    }

    contents.replace("::", "_")
}

/// Include shader source code & validate at compile time. Path must be relative to the crate root.
///
/// # Example
///
/// ```
/// let shader_str = include_wgsl!("src/shader.wgsl");
/// device.create_shader_module(&ShaderModuleDescriptor {
///     source: ShaderSource::Wgsl(Cow::Borrowed(&shader_str)),
///     flags: ShaderFlags::default(),
///     label: None,
/// })
/// ```
#[proc_macro]
pub fn include_wgsl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let file_path = input.value();

    let resolved_file_path = resolve_shader_path(&file_path);

    match std::fs::read_to_string(resolved_file_path.clone()) {
        Ok(wgsl_str) => {
            // Resolve shader includes
            let wgsl_str = parse_shader_includes(wgsl_str);

            // Attempt to parse WGSL
            match wgsl::parse_str(&wgsl_str) {
                Ok(module) => {
                    // Attempt to validate WGSL
                    match Validator::new(ValidationFlags::all(), Capabilities::all())
                        .validate(&module)
                    {
                        Ok(_) => {}
                        Err(e) => {
                            return syn::Error::new(input.span(), format!("{}: {}", file_path, e))
                                .to_compile_error()
                                .into();
                        }
                    }
                }
                Err(e) => {
                    return syn::Error::new(input.span(), format!("{}: {}", file_path, e))
                        .to_compile_error()
                        .into();
                }
            }

            let resolved_file_path = resolved_file_path.to_str().unwrap().replace("\\", "/");

            format!(
                "{{ std::hint::black_box(include_str!(\"{}\")); r#\"{}\"# }}",
                resolved_file_path, wgsl_str
            )
            .parse()
            .unwrap()
        }
        Err(e) => syn::Error::new(input.span(), format!("{}: {}", file_path, e))
            .to_compile_error()
            .into(),
    }
}
