use super::shader_module_source_map;
use crate::plugin::ShaderModuleSourceBinding;

#[test]
fn shader_module_source_map_reports_same_token_from_distinct_owners() {
    let project = ShaderModuleSourceBinding::new(
        "package:one",
        "zircon_fixture::lighting",
        "fn fixture_lighting() -> vec3f { return vec3f(0.5); }",
        "fixture package one",
    );
    let duplicate = ShaderModuleSourceBinding::new(
        "package:two",
        "zircon_fixture::lighting",
        project.source.clone(),
        "fixture package two",
    );

    let error = shader_module_source_map([project, duplicate])
        .expect_err("distinct shader-module owners must remain diagnosable");
    assert!(error.to_string().contains("fixture package one"));
    assert!(error.to_string().contains("fixture package two"));
}
