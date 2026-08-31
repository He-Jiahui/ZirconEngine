const PROCEDURAL_SKY_SHADER: &str =
    include_str!("../src/graphics/shader/wgsl/zr_procedural_sky.wgsl");
const ENVIRONMENT_SHADER: &str = concat!(
    include_str!("../src/graphics/shader/wgsl/zr_procedural_sky.wgsl"),
    "\n",
    include_str!("../src/graphics/shader/wgsl/zr_environment_core.wgsl"),
    "\n",
    include_str!("../src/graphics/shader/wgsl/zr_environment_generic_api.wgsl"),
    "\n",
    include_str!("../src/graphics/shader/wgsl/zr_environment.wgsl"),
);
const SKYBOX_SHADER: &str = concat!(
    include_str!("../src/graphics/shader/wgsl/zr_procedural_sky.wgsl"),
    "\n",
    include_str!("../src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl"),
);
const REALTIME_CAPTURE_SHADER: &str = concat!(
    include_str!("../src/graphics/shader/wgsl/zr_procedural_sky.wgsl"),
    "\n",
    include_str!(
        "../src/graphics/scene/scene_renderer/environment/shaders/realtime_ibl_capture.wgsl"
    ),
);
const SKYBOX_SETTINGS_SOURCE: &str =
    include_str!("../src/core/framework/render/environment/skybox.rs");

fn environment_pbr_composition_source() -> String {
    function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_pbr_components_from_reflection(",
    )
}

enum WgslComment {
    NotComment,
    End(usize),
    UnterminatedBlock,
}

fn wgsl_line_break_len_at(bytes: &[u8], index: usize) -> Option<usize> {
    match bytes.get(index) {
        Some(b'\n' | b'\x0B' | b'\x0C') => Some(1),
        Some(b'\r') => Some(if bytes.get(index + 1) == Some(&b'\n') {
            2
        } else {
            1
        }),
        Some(&0xC2) if bytes.get(index + 1) == Some(&0x85) => Some(2),
        Some(&0xE2)
            if bytes.get(index + 1) == Some(&0x80)
                && matches!(bytes.get(index + 2), Some(0xA8 | 0xA9)) =>
        {
            Some(3)
        }
        _ => None,
    }
}

fn is_wgsl_blankspace_character(character: char) -> bool {
    matches!(
        character,
        ' ' | '\t'
            | '\n'
            | '\u{000B}'
            | '\u{000C}'
            | '\r'
            | '\u{0085}'
            | '\u{200E}'
            | '\u{200F}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

fn wgsl_blankspace_len_at(source: &str, index: usize) -> Option<usize> {
    let character = source.get(index..)?.chars().next()?;
    is_wgsl_blankspace_character(character).then_some(character.len_utf8())
}

fn wgsl_comment_at(bytes: &[u8], start: usize) -> WgslComment {
    if bytes.get(start) != Some(&b'/') {
        return WgslComment::NotComment;
    }

    match bytes.get(start + 1) {
        Some(&b'/') => {
            let mut index = start + 2;
            while index < bytes.len() && wgsl_line_break_len_at(bytes, index).is_none() {
                index += 1;
            }
            WgslComment::End(index)
        }
        Some(&b'*') => {
            let mut depth = 1usize;
            let mut index = start + 2;
            while index + 1 < bytes.len() {
                match (bytes[index], bytes[index + 1]) {
                    (b'/', b'*') => {
                        depth += 1;
                        index += 2;
                    }
                    (b'*', b'/') => {
                        depth -= 1;
                        index += 2;
                        if depth == 0 {
                            return WgslComment::End(index);
                        }
                    }
                    _ => index += 1,
                }
            }
            WgslComment::UnterminatedBlock
        }
        _ => WgslComment::NotComment,
    }
}

fn wgsl_function_body_start(bytes: &[u8], signature: &str) -> usize {
    let mut index = 0;
    while index < bytes.len() {
        match wgsl_comment_at(bytes, index) {
            WgslComment::End(next) => {
                index = next;
                continue;
            }
            WgslComment::UnterminatedBlock => {
                panic!("unterminated block comment before {signature} body");
            }
            WgslComment::NotComment => {}
        }

        if bytes[index] == b'{' {
            return index;
        }
        index += 1;
    }

    panic!("missing opening brace for {signature}");
}

fn wgsl_code_position(bytes: &[u8], needle: &[u8]) -> Option<usize> {
    let mut index = 0;
    while index < bytes.len() {
        match wgsl_comment_at(bytes, index) {
            WgslComment::End(next) => {
                index = next;
                continue;
            }
            WgslComment::UnterminatedBlock => return None,
            WgslComment::NotComment => {}
        }

        if bytes[index..].starts_with(needle) {
            return Some(index);
        }
        index += 1;
    }

    None
}

fn wgsl_code_view(source: &str, context: &str) -> String {
    let bytes = source.as_bytes();
    let mut code = String::with_capacity(source.len());
    let mut index = 0;
    while index < bytes.len() {
        match wgsl_comment_at(bytes, index) {
            WgslComment::End(next) => {
                // Comments separate WGSL tokens, so preserve that boundary in the contract view.
                code.push(' ');
                index = next;
                continue;
            }
            WgslComment::UnterminatedBlock => {
                panic!("unterminated block comment in {context}");
            }
            WgslComment::NotComment => {}
        }

        let character = source[index..]
            .chars()
            .next()
            .expect("source index must remain on a UTF-8 boundary");
        code.push(character);
        index += character.len_utf8();
    }
    code
}

fn function_body(source: &str, signature: &str) -> String {
    let start = wgsl_code_position(source.as_bytes(), signature.as_bytes())
        .unwrap_or_else(|| panic!("missing {signature}"));
    let function = &source[start..];
    let body_start = wgsl_function_body_start(function.as_bytes(), signature);
    let mut depth = 1usize;
    let bytes = function.as_bytes();
    let mut index = body_start + 1;
    while index < bytes.len() {
        match wgsl_comment_at(bytes, index) {
            WgslComment::End(next) => {
                index = next;
                continue;
            }
            WgslComment::UnterminatedBlock => {
                panic!("unterminated block comment for {signature}");
            }
            WgslComment::NotComment => {}
        }

        match bytes[index] {
            b'{' => {
                depth += 1;
                index += 1;
            }
            b'}' => {
                depth = depth
                    .checked_sub(1)
                    .unwrap_or_else(|| panic!("unbalanced braces for {signature}"));
                if depth == 0 {
                    return wgsl_code_view(&function[body_start + 1..index], signature);
                }
                index += 1;
            }
            _ => index += 1,
        }
    }

    panic!("missing closing brace for {signature}");
}

fn is_wgsl_identifier_character(character: char) -> bool {
    character == '_'
        || character.is_ascii_alphanumeric()
        // WGSL's valid non-ASCII code points are XID identifier characters unless blankspace.
        || (!character.is_ascii() && !is_wgsl_blankspace_character(character))
}

fn is_wgsl_identifier_before(source: &str, index: usize) -> bool {
    source[..index]
        .chars()
        .next_back()
        .is_some_and(is_wgsl_identifier_character)
}

fn is_wgsl_identifier_after(source: &str, index: usize) -> bool {
    wgsl_blankspace_len_at(source, index).is_none()
        && source[index..]
            .chars()
            .next()
            .is_some_and(is_wgsl_identifier_character)
}

fn contains_wgsl_function_call(source: &str, function_name: &str) -> bool {
    let bytes = source.as_bytes();
    let name = function_name.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match wgsl_comment_at(bytes, index) {
            WgslComment::End(next) => {
                index = next;
                continue;
            }
            WgslComment::UnterminatedBlock => return false,
            WgslComment::NotComment => {}
        }

        let name_end = index + name.len();
        if bytes[index..].starts_with(name)
            && !is_wgsl_identifier_before(source, index)
            && !is_wgsl_identifier_after(source, name_end)
        {
            let mut call_start = name_end;
            loop {
                while let Some(blankspace_len) = wgsl_blankspace_len_at(source, call_start) {
                    call_start += blankspace_len;
                }
                match wgsl_comment_at(bytes, call_start) {
                    WgslComment::End(next) => call_start = next,
                    WgslComment::UnterminatedBlock => break,
                    WgslComment::NotComment => break,
                }
            }
            if bytes.get(call_start) == Some(&b'(') {
                return true;
            }
        }

        index += 1;
    }

    false
}

#[test]
fn function_body_keeps_nested_block_contents() {
    let body = function_body(
        "fn nested() { if (ready) { return; } let sentinel = 1; }",
        "fn nested()",
    );

    assert!(
        body.contains("let sentinel = 1;"),
        "function extraction must retain statements after nested blocks"
    );
}

#[test]
fn function_body_ignores_braces_in_comments() {
    let body = function_body(
        "fn commented() /* prelude { */ { // }\n /* outer { /* nested } */ still outer } */ let sentinel = 1; }",
        "fn commented()",
    );

    assert!(
        body.contains("let sentinel = 1;"),
        "function extraction must ignore braces inside comments"
    );
}

#[test]
fn function_body_ignores_commented_out_signatures() {
    let body = function_body(
        "// fn commented() { stale; }\nfn commented() { let sentinel = 1; }",
        "fn commented()",
    );

    assert!(
        body.contains("let sentinel = 1;"),
        "function extraction must skip commented-out declarations"
    );
}

#[test]
fn function_body_ignores_nested_block_comment_signatures() {
    let body = function_body(
        "/* outer { /* inner */ fn commented() { stale; } still outer } */ fn commented() { let sentinel = 1; }",
        "fn commented()",
    );

    assert!(
        body.contains("let sentinel = 1;"),
        "function extraction must skip nested block-comment declarations"
    );
}

#[test]
fn function_body_ends_line_comments_at_each_wgsl_line_break() {
    for line_break in [
        "\r", "\u{000B}", "\u{000C}", "\u{0085}", "\u{2028}", "\u{2029}",
    ] {
        let source = format!(
            "// fn commented() {{ stale; }}{line_break}fn commented() {{ let sentinel = 1; }}"
        );
        let body = function_body(&source, "fn commented()");
        assert!(
            body.contains("let sentinel = 1;"),
            "line comments must end at the WGSL line break {line_break:?}"
        );
    }
}

#[test]
fn function_body_excludes_commented_out_rotation_fast_path() {
    let body = function_body(
        "fn rotation() { /* if (scene.environment_rotation_sin_cos.z < 0.5) { return direction; } let s = scene.environment_rotation_sin_cos.x; */ let live = 1; }",
        "fn rotation()",
    );

    assert!(!body.contains("if (scene.environment_rotation_sin_cos.z < 0.5)"));
    assert!(!body.contains("return direction;"));
    assert!(!body.contains("let s = scene.environment_rotation_sin_cos.x;"));
    assert!(body.contains("let live = 1;"));
}

#[test]
#[should_panic(expected = "unterminated block comment")]
fn function_body_rejects_unterminated_block_comments() {
    let _ = function_body("fn incomplete() { /*", "fn incomplete()");
}

#[test]
fn wgsl_function_call_scan_accepts_whitespace_but_not_identifiers_or_comments() {
    assert!(contains_wgsl_function_call("sin (value)", "sin"));
    assert!(contains_wgsl_function_call("cos\t(value)", "cos"));
    assert!(contains_wgsl_function_call("sin/* note */(value)", "sin"));
    assert!(contains_wgsl_function_call("cos // note\n(value)", "cos"));
    assert!(contains_wgsl_function_call("sin\u{200E}(value)", "sin"));
    assert!(contains_wgsl_function_call(
        "sin/* outer /* nested */ note */ (value)",
        "sin"
    ));
    assert!(!contains_wgsl_function_call(
        "let sin_cos = 1.0; // sin (value)",
        "sin"
    ));
    assert!(!contains_wgsl_function_call("\u{0394}sin (value)", "sin"));
    assert!(!contains_wgsl_function_call(
        "let result = asin (value);",
        "sin"
    ));
}

#[test]
fn runtime_environment_wgpu_cubemap_sampling_does_not_warp_lookup_directions() {
    for (label, source, signature) in [
        (
            "environment",
            ENVIRONMENT_SHADER,
            "fn zr_environment_fix_cube_lookup_for_face_size",
        ),
        ("skybox", SKYBOX_SHADER, "fn skybox_fix_cube_lookup"),
    ] {
        let body = function_body(source, signature);
        assert!(
            body.contains("return direction;"),
            "{label} must preserve the cube lookup direction"
        );
        assert!(
            !body.contains("adjusted"),
            "{label} must not retain legacy edge-warp state"
        );
        assert!(
            !body.contains("exp2("),
            "{label} must not retain legacy LOD edge warping"
        );
    }
}

#[test]
fn runtime_environment_precomputed_rotation_skips_per_pixel_trigonometry() {
    let rotation = function_body(ENVIRONMENT_SHADER, "fn zr_environment_rotated_direction(");

    let zero_guard = rotation
        .find("if (scene.environment_rotation_sin_cos.z < 0.5)")
        .expect("the default environment rotation must retain the precomputed uniform fast path");
    let identity_return = rotation[zero_guard..]
        .find("return direction;")
        .map(|offset| zero_guard + offset)
        .expect("zero rotation must preserve the already-normalized direction");
    let sine = rotation
        .find("let s = scene.environment_rotation_sin_cos.x;")
        .expect("a rotated environment must consume its precomputed sine");
    let cosine = rotation
        .find("let c = scene.environment_rotation_sin_cos.y;")
        .expect("a rotated environment must consume its precomputed cosine");

    assert!(
        zero_guard < identity_return && identity_return < sine && sine < cosine,
        "the default environment rotation must return before reading precomputed sine/cosine"
    );
    for forbidden in ["sin", "cos"] {
        assert!(
            !contains_wgsl_function_call(&rotation, forbidden),
            "environment rotation must not evaluate per-pixel trigonometry through `{forbidden}`"
        );
    }
}

#[test]
fn runtime_environment_cpu_sun_rotation_is_inverse_of_shader_lookup_rotation() {
    let environment_rotation =
        function_body(ENVIRONMENT_SHADER, "fn zr_environment_rotated_direction(");
    let skybox_rotation = function_body(SKYBOX_SHADER, "fn skybox_rotated_direction_normalized(");
    for (label, rotation) in [
        ("environment", environment_rotation),
        ("skybox", skybox_rotation),
    ] {
        assert!(
            rotation.contains("direction.x * c - direction.z * s"),
            "{label} lookup rotation must retain its positive-angle x row"
        );
        assert!(
            rotation.contains("direction.x * s + direction.z * c"),
            "{label} lookup rotation must retain its positive-angle z row"
        );
    }

    let cpu_rotation = SKYBOX_SETTINGS_SOURCE
        .split("fn direction_for_sampling_rotation(")
        .nth(1)
        .and_then(|source| source.split("impl ProceduralSkyParams").next())
        .expect("procedural sky should retain the CPU sampling-rotation owner");
    assert!(cpu_rotation.contains("self.direction.x * cosine + self.direction.z * sine"));
    assert!(cpu_rotation.contains("-self.direction.x * sine + self.direction.z * cosine"));
}

#[test]
fn runtime_environment_skybox_reuses_reconstructed_normalized_direction() {
    let rotation = function_body(SKYBOX_SHADER, "fn skybox_rotated_direction_normalized(");
    assert!(
        !rotation.contains("normalize("),
        "the skybox rotation owner must not renormalize its reconstructed unit direction"
    );
    let zero_guard = rotation
        .find("if (scene.environment_rotation_sin_cos.z < 0.5)")
        .expect("the default skybox rotation must retain the precomputed uniform fast path");
    let identity_return = rotation[zero_guard..]
        .find("return direction;")
        .map(|offset| zero_guard + offset)
        .expect("zero skybox rotation must preserve the reconstructed direction");
    let sine = rotation
        .find("let s = scene.environment_rotation_sin_cos.x;")
        .expect("a rotated skybox must consume its precomputed sine");
    let cosine = rotation
        .find("let c = scene.environment_rotation_sin_cos.y;")
        .expect("a rotated skybox must consume its precomputed cosine");
    assert!(
        zero_guard < identity_return && identity_return < sine && sine < cosine,
        "the default skybox rotation must return before reading precomputed sine/cosine"
    );
    for forbidden in ["sin", "cos"] {
        assert!(
            !contains_wgsl_function_call(&rotation, forbidden),
            "skybox rotation must not evaluate per-pixel trigonometry through `{forbidden}`"
        );
    }

    let source_sample = function_body(SKYBOX_SHADER, "fn source_cubemap_sky_color(");
    assert!(
        contains_wgsl_function_call(&source_sample, "skybox_rotated_direction_normalized")
            && source_sample.contains("skybox_rotated_direction_normalized(direction)"),
        "source cubemap sky sampling must use the normalized rotation owner"
    );
}

#[test]
fn runtime_environment_source_cubemap_reflections_use_pmrem_before_procedural_fallback() {
    let reflection = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_sky_reflection_color(",
    );

    for expected in [
        "if (zr_environment_is_source_cubemap() || zr_environment_is_realtime_ibl())",
        "let lod = zr_environment_mip_from_roughness(roughness, max_mip);",
        "return zr_environment_specular_pmrem_color_at_clamped_lod_normalized(reflected, lod);",
    ] {
        assert!(
            reflection.contains(expected),
            "source/realtime IBL reflection must use PMREM through `{expected}`"
        );
    }
    let fallback = reflection
        .split(
            "return zr_environment_specular_pmrem_color_at_clamped_lod_normalized(reflected, lod);",
        )
        .nth(1)
        .expect("procedural fallback should remain separate from source/realtime IBL");
    assert!(
        fallback.contains("return zr_environment_procedural_sky_color_normalized(reflected);"),
        "a procedural fallback without PMREM must preserve the reflected direction"
    );
    assert_eq!(
        fallback
            .matches("zr_environment_procedural_sky_color_normalized(")
            .count(),
        1,
        "the fallback must contain exactly one sky lookup"
    );
    assert_eq!(
        fallback
            .matches("zr_environment_procedural_sky_color_normalized(reflected)")
            .count(),
        1,
        "the sole fallback sky lookup must preserve the reflected direction"
    );
    for forbidden in [
        "zr_environment_procedural_sky_color_normalized(normal)",
        "mix(sharp_reflection, rough_reflection, roughness)",
    ] {
        assert!(
            !fallback.contains(forbidden),
            "a procedural fallback without PMREM must not synthesize roughness with `{forbidden}`"
        );
    }
}

#[test]
fn runtime_environment_procedural_pbr_reuses_normalized_directions() {
    let normalized_sky = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_procedural_sky_color_normalized(",
    );
    assert!(
        !normalized_sky.contains("zr_environment_normalize_or_zero("),
        "the normalized procedural-sky path must not normalize its input a second time"
    );

    let defensive_sky = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_procedural_sky_color(",
    );
    assert!(
        contains_wgsl_function_call(
            &defensive_sky,
            "zr_environment_procedural_sky_color_normalized",
        ) && defensive_sky.contains("zr_environment_normalize_or_zero(direction)"),
        "the public procedural-sky wrapper must retain defensive normalization"
    );

    let diffuse = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_diffuse_color_normalized(",
    );
    assert!(
        contains_wgsl_function_call(&diffuse, "zr_environment_procedural_sky_color_normalized")
            && diffuse.contains("zr_environment_procedural_sky_color_normalized(normal)"),
        "procedural diffuse must reuse its normalized PBR normal"
    );
}

#[test]
fn runtime_environment_procedural_sky_uses_shared_source_radiance_owner() {
    let helper = function_body(PROCEDURAL_SKY_SHADER, "fn zr_procedural_sky_radiance(");
    assert!(
        !helper.contains("normalize("),
        "the shared procedural-sky helper must consume a normalized direction"
    );
    for forbidden in ["sin(", "cos("] {
        assert!(
            !contains_wgsl_function_call(&helper, forbidden.trim_end_matches('(')),
            "the shared procedural-sky helper must not calculate {forbidden} per invocation"
        );
    }

    for (label, source, signature) in [
        (
            "environment",
            ENVIRONMENT_SHADER,
            "fn zr_environment_procedural_sky_color_normalized(",
        ),
        ("skybox", SKYBOX_SHADER, "fn fs_main("),
        ("realtime capture", REALTIME_CAPTURE_SHADER, "fn cs_main("),
    ] {
        let consumer = function_body(source, signature);
        assert!(
            contains_wgsl_function_call(&consumer, "zr_procedural_sky_radiance"),
            "{label} must call the shared source-radiance owner"
        );
        assert_eq!(
            source.matches("fn zr_procedural_sky_radiance(").count(),
            1,
            "{label} pipeline must assemble one shared procedural-sky radiance owner"
        );
    }
}

#[test]
fn runtime_environment_procedural_sun_uses_cpu_prepared_parameters() {
    let shared_radiance = function_body(PROCEDURAL_SKY_SHADER, "fn zr_procedural_sky_radiance(");

    assert!(
        shared_radiance.contains("sun_direction.xyz"),
        "the shared source-radiance owner must consume the CPU-normalized sun direction"
    );
    for source in [shared_radiance] {
        assert!(
            !source.contains("sun_direction_length"),
            "the shared source-radiance owner must not measure a uniform direction per invocation"
        );
        assert!(
            !source.contains("angular_radius"),
            "the shared source-radiance owner must consume precomputed cosine edges"
        );
        assert!(
            !source.contains("cos("),
            "the shared source-radiance owner must not compute sun cosine edges per invocation"
        );
    }
}

#[test]
fn runtime_environment_direct_procedural_sun_obeys_final_sampling_intensity() {
    let environment_sun = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_procedural_sky_color_normalized(",
    );
    assert!(environment_sun.contains("zr_procedural_sky_radiance("));
    assert!(environment_sun.contains("* max(scene.environment_params.y, 0.0);"));

    let skybox_fragment = function_body(SKYBOX_SHADER, "fn fs_main(");
    let color_composition = skybox_fragment
        .find("color = zr_procedural_sky_radiance(")
        .expect("the procedural skybox must use the shared source-radiance owner");
    let final_intensity = skybox_fragment[color_composition..]
        .find("* max(scene.environment_params.y, 0.0);")
        .map(|offset| color_composition + offset)
        .expect("the procedural skybox must apply final sampling intensity");
    assert!(
        color_composition < final_intensity,
        "the shared procedural sun and sky radiance must receive the same final sampling intensity"
    );
}

#[test]
fn runtime_environment_pbr_pmrem_reuses_normalized_direction_and_clamped_lod() {
    let normalized_sample = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_specular_pmrem_color_at_clamped_lod_normalized",
    );
    assert!(
        normalized_sample.contains("zr_environment_rotated_direction(direction)"),
        "the normalized PMREM path must still apply runtime environment rotation"
    );
    for forbidden in [
        "zr_environment_normalize_or_zero(",
        "scene.environment_sample_params.w",
        "clamp(lod",
    ] {
        assert!(
            !normalized_sample.contains(forbidden),
            "the normalized PMREM path must reuse prepared input instead of `{forbidden}`"
        );
    }

    let defensive_sample = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_specular_pmrem_color_at_lod",
    );
    for expected in [
        "zr_environment_normalize_or_zero(direction)",
        "let clamped_lod = clamp(lod, 0.0, max_mip);",
        "zr_environment_specular_pmrem_color_at_clamped_lod_normalized(",
    ] {
        assert!(
            defensive_sample.contains(expected),
            "the public PMREM wrapper must retain defensive preparation through `{expected}`"
        );
    }
}

#[test]
fn runtime_environment_planar_reflection_short_circuits_pmrem_and_probe_work() {
    let normalized_reflection = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_reflection_color_normalized(",
    );
    let reflection = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_reflection_color_after_planar(",
    );

    let planar = normalized_reflection
        .find("let planar = zr_environment_planar_reflection(")
        .expect("reflection must evaluate the planar candidate");
    let planar_return = normalized_reflection[planar..]
        .find("return planar.rgb;")
        .map(|offset| planar + offset)
        .expect("a valid planar candidate must return before lower-priority reflection work");
    let continuation = normalized_reflection[planar_return..]
        .find("zr_environment_reflection_color_after_planar(")
        .map(|offset| planar_return + offset)
        .expect("the non-planar path must continue after the planar return");
    let reflected = reflection
        .find("let reflected = reflect(-view_dir, normal);")
        .expect("reflection must retain the reflected lookup direction");
    let sky = reflection
        .find("var sky = vec3<f32>(0.0);")
        .expect("the probe path must retain conditional sky storage");
    let probes = reflection
        .find("let selection = zr_environment_select_probes(")
        .expect("reflection must retain probe selection");
    let no_probes = reflection
        .find("if (zr_env_probe_header.probe_count == 0u)")
        .expect("reflection must skip probe selection when no probes exist");
    let no_probe_return = reflection[no_probes..]
        .find("return zr_environment_sky_reflection_color(reflected, clamped_roughness);")
        .map(|offset| no_probes + offset)
        .expect("the no-probe fast path must return sky reflection before probe selection");

    assert!(
        planar < planar_return
            && planar_return < continuation
            && reflected < no_probes
            && no_probes < no_probe_return
            && no_probe_return < probes
            && probes < sky,
        "planar reflection must short-circuit reflected-direction, PMREM sampling, and probe selection"
    );
}

#[test]
fn runtime_environment_full_probe_coverage_skips_zero_weight_sky_sample() {
    let reflection = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_reflection_color_after_planar(",
    );

    let no_probes = reflection
        .find("if (zr_env_probe_header.probe_count == 0u)")
        .expect("the no-probe sky fast path must remain explicit");
    let selection = reflection
        .find("let selection = zr_environment_select_probes(")
        .expect("reflection must select probes after the no-probe fast path");
    let sky_weight = reflection
        .find("let sky_weight = max(")
        .expect("reflection must retain explicit sky weighting");
    let sky_guard = reflection
        .find("if (sky_weight > 0.0")
        .expect("zero-weight sky sampling must be guarded");
    let sky_sample = reflection[sky_guard..]
        .find("zr_environment_sky_reflection_color(")
        .map(|offset| sky_guard + offset)
        .expect("a positive sky weight must retain the sky sample");

    assert!(
        no_probes < selection
            && selection < sky_weight
            && sky_weight < sky_guard
            && sky_guard < sky_sample,
        "fully probe-covered pixels must not sample zero-weight sky PMREM"
    );
}

#[test]
fn runtime_environment_pbr_reuses_normalized_reflection_inputs() {
    let components = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_pbr_components_with_prepared_inputs(",
    );

    assert!(
        contains_wgsl_function_call(&components, "zr_environment_reflection_color_normalized"),
        "prepared PBR components must call the normalized reflection hot path"
    );
    assert!(
        !components.contains("zr_environment_reflection_color(\n"),
        "prepared PBR components must not repeat normalization before reflection"
    );

    let wrapper = function_body(ENVIRONMENT_SHADER, "fn zr_environment_reflection_color(");
    for expected in [
        "zr_environment_normalize_or_zero(normal_ws)",
        "zr_environment_normalize_or_zero(view_dir_ws)",
        "clamp(roughness, 0.0, 1.0)",
    ] {
        assert!(
            wrapper.contains(expected),
            "the defensive reflection wrapper must retain `{expected}`"
        );
    }

    let planar = wrapper
        .find("let planar = zr_environment_planar_reflection(")
        .expect("the defensive wrapper must evaluate planar reflection before normalizing inputs");
    let planar_return = wrapper[planar..]
        .find("return planar.rgb;")
        .map(|offset| planar + offset)
        .expect("a valid planar candidate must return from the defensive wrapper");
    let normal = wrapper
        .find("zr_environment_normalize_or_zero(normal_ws)")
        .expect("the wrapper must retain normal normalization for non-planar reflections");
    let view = wrapper
        .find("zr_environment_normalize_or_zero(view_dir_ws)")
        .expect("the wrapper must retain view normalization for non-planar reflections");
    assert!(
        planar < planar_return && planar_return < normal && normal < view,
        "planar reflection must avoid defensive normal/view normalization"
    );
}

#[test]
fn runtime_environment_full_metal_skips_zero_weight_diffuse_ibl() {
    let components = environment_pbr_composition_source();

    let no_v = components
        .find("let no_v = clamp(dot(normal, view_dir), 0.0, 1.0);")
        .expect("PBR indirect must derive NdotV before Fresnel energy conservation");
    let f0 = components
        .find("let f0 = zr_pbr_material_f0(dielectric_f0, base_color, clamped_metallic);")
        .expect("PBR indirect must derive material F0 before diffuse energy scaling");
    let diffuse_energy = components
        .find("let diffuse_energy_scale = zr_pbr_diffuse_energy_scale(")
        .expect("PBR indirect must retain Fresnel-aware diffuse energy scaling");
    let guard = components
        .find("if (any(diffuse_energy_scale > vec3<f32>(0.0))")
        .expect("zero-weight diffuse IBL must be guarded for full-metal materials");
    let diffuse_sample = components
        .find("zr_environment_diffuse_color_normalized(normal)")
        .expect("PBR indirect must retain diffuse IBL below the metallic guard");

    assert!(
        no_v < f0 && f0 < diffuse_energy && diffuse_energy < guard && guard < diffuse_sample,
        "full-metal materials must not evaluate diffuse IBL before its zero-weight guard"
    );
    assert!(components.contains("zr_pbr_fresnel_schlick(no_v, f0),"));
}

#[test]
fn runtime_environment_pbr_diffuse_reuses_its_normalized_normal() {
    let normalized_diffuse = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_diffuse_color_normalized(",
    );

    assert!(
        contains_wgsl_function_call(&normalized_diffuse, "zr_environment_sh9_color_normalized")
            && normalized_diffuse.contains("zr_environment_sh9_color_normalized(normal)"),
        "normalized diffuse IBL must pass its normalized normal through to SH9"
    );
    assert!(
        contains_wgsl_function_call(
            &normalized_diffuse,
            "zr_environment_irradiance_cube_color_normalized",
        ) && normalized_diffuse.contains("zr_environment_irradiance_cube_color_normalized(normal)"),
        "normalized diffuse IBL must pass its normalized normal through to the irradiance cube"
    );
    assert!(
        !normalized_diffuse.contains("zr_environment_normalize_or_zero("),
        "the PBR normalized diffuse path must not normalize the normal a second time"
    );
}

#[test]
fn runtime_environment_sh9_diffuse_tracks_runtime_sky_rotation() {
    let normalized_diffuse = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_diffuse_color_normalized(",
    );

    assert_eq!(
        normalized_diffuse
            .matches("zr_environment_sh9_color_normalized(normal)")
            .count(),
        2,
        "source and realtime SH9 diffuse must share the rotation-aware normalized owner"
    );

    let rotated_sh9 = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_sh9_color_normalized(",
    );
    let rotation = rotated_sh9
        .find("let rotated = zr_environment_rotated_direction(normal);")
        .expect("SH9 diffuse must rotate the normalized world normal with the runtime sky");
    let evaluation = rotated_sh9
        .find("zr_environment_sh9_eval_normalized(rotated)")
        .expect("SH9 diffuse must evaluate coefficients in the rotated environment direction");

    assert!(
        rotation < evaluation,
        "runtime sky rotation must be applied before SH9 coefficient evaluation"
    );
    assert!(
        !rotated_sh9.contains("zr_environment_normalize_or_zero("),
        "the rotation-aware SH9 path must reuse the PBR-normalized normal"
    );
}
