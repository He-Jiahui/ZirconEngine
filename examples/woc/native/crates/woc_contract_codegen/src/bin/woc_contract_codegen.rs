use std::path::PathBuf;

use woc_contract_codegen::{
    generate_projections, load_contract_manifest, verify_projection, write_projection,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let manifest = required_path(&mut arguments, "contract manifest")?;
    let rust_output = required_path(&mut arguments, "Rust output")?;
    let zrvm_output = required_path(&mut arguments, "ZrVM output")?;
    let check = arguments.next().as_deref() == Some(std::ffi::OsStr::new("--check"));
    if arguments.next().is_some() {
        return Err("unexpected extra arguments".into());
    }

    let manifest = load_contract_manifest(manifest)?;
    let projections = generate_projections(&manifest)?;
    if check {
        verify_projection(rust_output, &projections.rust)?;
        verify_projection(zrvm_output, &projections.zrvm)?;
    } else {
        write_projection(rust_output, &projections.rust)?;
        write_projection(zrvm_output, &projections.zrvm)?;
    }
    println!("{}", projections.fingerprint_hex);
    Ok(())
}

fn required_path(
    arguments: &mut impl Iterator<Item = std::ffi::OsString>,
    label: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing {label}").into())
}
