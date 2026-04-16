fn main() {
    println!("cargo:rerun-if-changed=ui/workbench.slint");
    println!("cargo:rerun-if-changed=ui/workbench");
    println!("cargo:rerun-if-changed=ui/templates");
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent".into())
        .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles);
    slint_build::compile_with_config("ui/workbench.slint", config).expect("compile Slint UI");
    write_viewport_gizmo_icon_manifest().expect("generate viewport gizmo icon manifest");
}

fn write_viewport_gizmo_icon_manifest() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let manifest_path = out_dir.join("viewport_gizmo_icon_manifest.rs");
    let icons = [
        ("Camera", "camera.pbm"),
        ("DirectionalLight", "directional_light.pbm"),
    ];

    println!("cargo:rerun-if-changed=assets/viewport_gizmos");
    for (_, filename) in icons {
        println!("cargo:rerun-if-changed=assets/viewport_gizmos/{filename}");
    }

    let mut generated = String::from(
        "pub(crate) fn viewport_gizmo_icon_bytes(id: ViewportIconId) -> Option<&'static [u8]> {\n    match id {\n",
    );
    for (variant, filename) in icons {
        generated.push_str(&format!(
            "        ViewportIconId::{variant} => Some(include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/viewport_gizmos/{filename}\"))),\n"
        ));
    }
    generated.push_str("    }\n}\n");
    fs::write(manifest_path, generated)?;
    Ok(())
}
