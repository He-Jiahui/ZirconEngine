use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};

use i_slint_compiler::diagnostics::BuildDiagnostics;
use i_slint_compiler::generator::OutputFormat;

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"));
    let input = manifest_dir.join("ui/app.slint");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set")).join("app.rs");

    let dependencies = compile_with_flexbox(&input, &output).expect("compile zircon_hub Slint UI");
    for dependency in dependencies {
        println!("cargo:rerun-if-changed={}", dependency.display());
    }
    println!("cargo:rerun-if-env-changed=SLINT_STYLE");
    println!("cargo:rerun-if-env-changed=SLINT_FONT_SIZES");
    println!("cargo:rerun-if-env-changed=SLINT_SCALE_FACTOR");
    println!("cargo:rerun-if-env-changed=SLINT_ASSET_SECTION");
    println!("cargo:rerun-if-env-changed=SLINT_EMBED_RESOURCES");
    println!("cargo:rerun-if-env-changed=SLINT_EMIT_DEBUG_INFO");
    println!("cargo:rerun-if-env-changed=SLINT_LIVE_PREVIEW");
    println!(
        "cargo:rustc-env=SLINT_INCLUDE_GENERATED={}",
        output.display()
    );
}

fn compile_with_flexbox(input: &Path, output: &Path) -> Result<Vec<PathBuf>, String> {
    let manifest_dir = input
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| format!("invalid Slint input path: {}", input.display()))?;
    let mut diagnostics = BuildDiagnostics::default();
    let syntax_node = i_slint_compiler::parser::parse_file(input, &mut diagnostics);
    if diagnostics.has_errors() {
        let messages = diagnostics.to_string_vec();
        diagnostics.print();
        return Err(messages.join("\n"));
    }

    let mut config = i_slint_compiler::CompilerConfiguration::new(OutputFormat::Rust);
    config.translation_domain = env::var("CARGO_PKG_NAME").ok();
    config.enable_experimental = true;
    config.library_paths.insert(
        "material".to_string(),
        manifest_dir
            .parent()
            .expect("zircon_hub manifest layout is stable")
            .join("dev/material-rust-template/material-1.0/material.slint"),
    );

    let syntax_node = syntax_node.expect("diagnostics contained no parser errors");
    let (document, diagnostics, loader) = spin_on::spin_on(i_slint_compiler::compile_syntax_node(
        syntax_node,
        diagnostics,
        config,
    ));

    if diagnostics.has_errors()
        || (!diagnostics.is_empty() && env::var("SLINT_COMPILER_DENY_WARNINGS").is_ok())
    {
        let messages = diagnostics.to_string_vec();
        diagnostics.print();
        return Err(messages.join("\n"));
    }

    let generated = i_slint_compiler::generator::rust::generate(&document, &loader.compiler_config)
        .map_err(|error| error.to_string())?;
    let generated = syn::parse2(generated)
        .map(|file| prettyplease::unparse(&file))
        .map_err(|error| error.to_string())?;
    let mut output_file = std::fs::File::create(output).map_err(|error| error.to_string())?;
    output_file
        .write_all(generated.as_bytes())
        .map_err(|error| error.to_string())?;

    let mut dependencies = Vec::new();
    for loaded in &diagnostics.all_loaded_files {
        if loaded.is_absolute() {
            dependencies.push(loaded.clone());
        }
    }
    dependencies.push(input.to_path_buf());
    for embedded in document.embedded_file_resources.borrow().iter() {
        if let Some(resource) = embedded.path.as_deref() {
            if !resource.starts_with("builtin:") {
                dependencies.push(Path::new(resource).to_path_buf());
            }
        }
    }
    Ok(dependencies)
}
