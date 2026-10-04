#![cfg(feature = "graphics")]

const CANONICAL_SCENE: &str = include_str!("../src/graphics/shader/wgsl/zr_scene_runtime.wgsl");
const SSAO_SHADERS: [(&str, &str); 3] = [
    (
        "evaluate",
        include_str!("../src/graphics/scene/scene_renderer/post_process/shaders/ssao.wgsl"),
    ),
    (
        "spatial",
        include_str!("../src/graphics/scene/scene_renderer/post_process/shaders/ssao_spatial_denoise.wgsl"),
    ),
    (
        "bilateral-upsample",
        include_str!("../src/graphics/scene/scene_renderer/post_process/shaders/ssao_bilateral_upsample.wgsl"),
    ),
];

#[test]
fn runtime_ssao_scene_uniform_prefix_matches_naga_layout() {
    let canonical = validated_module("canonical scene", CANONICAL_SCENE);
    let (expected_members, expected_span) = scene_uniform(&canonical);
    assert_eq!(expected_span, 496);
    for (name, offset) in [
        ("ambient_color", 192),
        ("lightmapped_ambient_color", 208),
        ("previous_view_proj_unjittered", 224),
    ] {
        assert_eq!(
            expected_members
                .iter()
                .find(|member| member.name.as_deref() == Some(name))
                .expect("canonical ABI field")
                .offset,
            offset,
            "{name} must match the CPU SceneUniform ABI"
        );
    }

    for (label, source) in SSAO_SHADERS {
        let module = validated_module(label, source);
        let (members, span) = scene_uniform(&module);
        assert!(!members.is_empty() && members.len() <= expected_members.len());
        for (actual, expected) in members.iter().zip(expected_members) {
            assert_eq!(actual.name, expected.name, "{label}: field order");
            assert_eq!(actual.offset, expected.offset, "{label}: byte offset");
            assert_eq!(
                module.types[actual.ty].inner, canonical.types[expected.ty].inner,
                "{label}: field type"
            );
        }
        assert_eq!(
            span,
            expected_members
                .get(members.len())
                .map_or(expected_span, |next| next.offset),
            "{label}: uniform prefix size"
        );
    }
}

fn validated_module(label: &str, source: &str) -> naga::Module {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("{label}: {}", error.emit_to_string(source)));
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .unwrap_or_else(|error| panic!("{label}: {error}"));
    module
}

fn scene_uniform(module: &naga::Module) -> (&[naga::StructMember], u32) {
    module
        .types
        .iter()
        .find_map(|(_, ty)| match (&ty.inner, ty.name.as_deref()) {
            (naga::TypeInner::Struct { members, span }, Some("SceneUniform")) => {
                Some((members.as_slice(), *span))
            }
            _ => None,
        })
        .expect("shader must declare SceneUniform")
}
