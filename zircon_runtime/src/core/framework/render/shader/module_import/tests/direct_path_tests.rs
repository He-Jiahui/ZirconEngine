use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{
    derive_shader_import_path, normalized_shader_asset_path, shader_import_namespace,
    strip_shader_asset_extension, ShaderImportPathDerivation, ShaderImportPathDerivationError,
};

const SAMPLE_PAIRS: usize = 101;
const PATHS_PER_SAMPLE: usize = 256;

#[test]
fn runtime875_shader_import_direct_path_preserves_derivations() {
    for (namespace, path) in [
        ("My Shader Project", "res://shaders/cloth/common.zshader"),
        (
            "12 Project",
            "assets\\Shaders\\02 cloth\\fold\\fold.wgsl#albedo",
        ),
        ("Project", "shaders/shaders.zshader"),
        ("project", "shaders/a/../_!/é-wgsl.wgsl"),
        ("project", "shaders/a/._.wgsl"),
        ("project", "shaders/a/a.wgsl"),
        ("project", "textures/a.zshader"),
        ("project", "shaders"),
        ("self", "shaders/valid.zshader"),
        ("project", "res:///"),
    ] {
        assert_eq!(
            derive_shader_import_path(namespace, path),
            legacy_derive(namespace, path),
            "namespace={namespace:?} path={path:?}"
        );
    }
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn runtime875_shader_import_direct_path_release_percentiles() {
    let path = format!(
        "res://shaders/{}/final.zshader",
        (0..31)
            .map(|index| format!("segment_{index:02}"))
            .collect::<Vec<_>>()
            .join("/")
    );
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&path, legacy_derive));
            optimized.push(measure(&path, derive_shader_import_path));
        } else {
            optimized.push(measure(&path, derive_shader_import_path));
            legacy.push(measure(&path, legacy_derive));
        }
    }
    let legacy_p50_ns = percentile(&mut legacy.clone(), 50);
    let legacy_p95_ns = percentile(&mut legacy.clone(), 95);
    let legacy_p99_ns = percentile(&mut legacy, 99);
    let optimized_p50_ns = percentile(&mut optimized.clone(), 50);
    let optimized_p95_ns = percentile(&mut optimized.clone(), 95);
    let optimized_p99_ns = percentile(&mut optimized, 99);
    println!(
        "RUNTIME875_SHADER_IMPORT_DIRECT_PATH_BENCH_V1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100));
}

fn legacy_derive(
    namespace: &str,
    path: &str,
) -> Result<ShaderImportPathDerivation, ShaderImportPathDerivationError> {
    let namespace = shader_import_namespace(namespace)?;
    let normalized = normalized_shader_asset_path(path)?;
    let root = normalized
        .iter()
        .position(|segment| segment.eq_ignore_ascii_case("shaders"))
        .ok_or_else(|| ShaderImportPathDerivationError::MissingShaderRoot {
            path: normalized.join("/"),
        })?;
    let mut module = normalized[root + 1..].to_vec();
    if module.is_empty() {
        return Err(ShaderImportPathDerivationError::EmptyModulePath {
            path: normalized.join("/"),
        });
    }
    if let Some(last) = module.last_mut() {
        *last = strip_shader_asset_extension(last).to_string();
    }
    if module.len() >= 2 && module[module.len() - 2].eq_ignore_ascii_case(&module[module.len() - 1])
    {
        module.pop();
    }
    if module.is_empty() {
        return Err(ShaderImportPathDerivationError::EmptyModulePath {
            path: normalized.join("/"),
        });
    }
    let mut result = Vec::with_capacity(module.len() + 1);
    result.push(namespace);
    for segment in module {
        let mut text = String::new();
        let mut previous_underscore = false;
        for ch in segment.chars() {
            if ch.is_ascii_alphanumeric() {
                text.push(ch.to_ascii_lowercase());
                previous_underscore = false;
            } else if !previous_underscore && !text.is_empty() {
                text.push('_');
                previous_underscore = true;
            }
        }
        while text.ends_with('_') {
            text.pop();
        }
        if text.is_empty() {
            return Err(ShaderImportPathDerivationError::EmptyModuleSegment {
                path: normalized.join("/"),
            });
        }
        if text.as_bytes()[0].is_ascii_digit() {
            text.insert(0, '_');
        }
        result.push(text);
    }
    Ok(ShaderImportPathDerivation {
        import_path: result.join("::"),
        folded_terminal_directory: super::terminal_directory_was_folded(&normalized),
    })
}

fn measure(
    path: &str,
    derive: fn(&str, &str) -> Result<ShaderImportPathDerivation, ShaderImportPathDerivationError>,
) -> Duration {
    let started = Instant::now();
    let checksum = (0..PATHS_PER_SAMPLE)
        .map(|_| {
            black_box(derive(black_box("Shader Project"), black_box(path)).unwrap())
                .import_path
                .len()
        })
        .sum::<usize>();
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percent: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)].as_nanos()
}
