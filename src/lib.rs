use naga::{
    front::wgsl,
    valid::{Capabilities, ValidationFlags, Validator},
};
use proc_macro::{Span, TokenStream};
use std::path::{Path, PathBuf};
use syn::{parse_macro_input, LitStr};

fn resolve_shader_path<T: Into<PathBuf>>(call_site: &Path, shader_path: T) -> PathBuf {
    call_site.join(shader_path.into())
}

/// Where the line starting at `s` ends: its first line break (`\n` or the `\r` of `\r\n`), or the end of the text.
fn end_of_line_idx(s: &str) -> usize {
    s.find(['\r', '\n']).unwrap_or(s.len())
}

fn parse_shader_includes_recursive(
    call_site: PathBuf,
    name: &str,
    includes: &mut Vec<String>,
) -> String {
    // Absolute, so the path also works in the `include_str!` emitted at the call site
    let file_path = std::path::absolute(resolve_shader_path(&call_site, name))
        .unwrap()
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
        let end_of_line = end_of_line_idx(&contents[include_index..]) + include_index;
        let include_name = PathBuf::from(contents[(include_index + 9)..end_of_line].to_owned());

        let call_site = call_site
            .join(include_name.clone())
            .parent()
            .unwrap()
            .to_path_buf();
        let include_name = include_name.file_name().unwrap().to_str().unwrap();

        for i in (include_index..end_of_line).rev() {
            contents.remove(i);
        }
        contents.insert_str(
            include_index,
            &parse_shader_includes_recursive(call_site, include_name, includes),
        );
    }

    contents
}

/// Resolves all includes, returns the final source and the paths of every included file.
fn parse_shader_includes(call_site: PathBuf, mut contents: String) -> (String, Vec<String>) {
    let mut includes = vec![];

    let mut include_indices: Vec<usize> = contents.match_indices("@include").map(|i| i.0).collect();
    include_indices.reverse();
    for include_index in include_indices {
        let end_of_line = end_of_line_idx(&contents[include_index..]) + include_index;
        let include_name = PathBuf::from(contents[(include_index + 9)..end_of_line].to_owned());

        let call_site = call_site
            .join(include_name.clone())
            .parent()
            .unwrap()
            .to_path_buf();
        let include_name = include_name.file_name().unwrap().to_str().unwrap();

        for i in (include_index..end_of_line).rev() {
            contents.remove(i);
        }
        contents.insert_str(
            include_index,
            &parse_shader_includes_recursive(call_site, include_name, &mut includes),
        );
    }

    (contents.replace("::", "_"), includes)
}

/// Include shader source code & validate at compile time. Path must be relative to the crate root.
///
/// # Example
///
/// ```ignore
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

    let call_site = Span::call_site()
        .local_file()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let resolved_file_path = resolve_shader_path(&call_site, &file_path);

    match std::fs::read_to_string(resolved_file_path.clone()) {
        Ok(wgsl_str) => {
            // Resolve shader includes
            let (wgsl_str, includes) = parse_shader_includes(
                PathBuf::from(resolved_file_path.parent().unwrap()),
                wgsl_str,
            );

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

            // `include_str!` every file that went into the shader, so editing any of them triggers a rebuild
            let mut tracked = format!("std::hint::black_box(include_str!({:?}));", file_path);
            for include in &includes {
                tracked += &format!("std::hint::black_box(include_str!({:?}));", include);
            }

            format!("{{ {} {:?} }}", tracked, wgsl_str).parse().unwrap()
        }
        Err(e) => syn::Error::new(input.span(), format!("{}: {}", file_path, e))
            .to_compile_error()
            .into(),
    }
}
