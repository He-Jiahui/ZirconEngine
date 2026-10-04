use super::*;

#[test]
fn parse_accepts_variants_flag() {
    let args = parse([
        OsString::from("--project-root"),
        OsString::from("project"),
        OsString::from("--out-dir"),
        OsString::from("out"),
        OsString::from("--variants"),
    ])
    .unwrap()
    .expect("parsed args");

    assert_eq!(args.project_root, PathBuf::from("project"));
    assert_eq!(args.output_dir, Some(PathBuf::from("out")));
    assert_eq!(
        args.preview_variants,
        vec![ShaderIdePreviewVariant::default_forward()]
    );
}

#[test]
fn parse_accepts_non_default_preview_variant_specs() {
    let args = parse([
        OsString::from("--variant"),
        OsString::from("gbuffer:options=0x1"),
        OsString::from("--variant"),
        OsString::from("shadow"),
        OsString::from("--variant"),
        OsString::from("hit_proxy"),
    ])
    .unwrap()
    .expect("parsed args");

    assert_eq!(
        args.preview_variants,
        vec![
            ShaderIdePreviewVariant::new(ShaderPassType::GBuffer, 1),
            ShaderIdePreviewVariant::new(ShaderPassType::Shadow, 0),
            ShaderIdePreviewVariant::new(ShaderPassType::HitProxy, 0),
        ]
    );
}
