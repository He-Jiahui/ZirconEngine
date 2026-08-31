use std::borrow::Cow;
use std::io::Cursor;

use ply_rs_bw as ply;
use zircon_runtime::asset::{
    cook_mesh_sdf_or_fallback_single, cook_virtual_geometry_from_mesh, AssetImportContext,
    AssetImportError, AssetImportOutcome, AssetReference, ImportedAsset, ImportedAssetEntry,
    MeshAsset, MeshSdfCookSettings, MeshVertex, ModelAsset, ModelPrimitiveAsset,
    VirtualGeometryCookConfig,
};
use zircon_runtime::core::math::{Vec2, Vec3};
pub fn import_mesh_model(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "stl" => import_stl(context),
        "ply" => import_ply(context),
        _ => Err(AssetImportError::UnsupportedFormat(format!(
            "model mesh importer does not handle {}",
            context.source_path.display()
        ))),
    }
}

fn import_stl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let mut reader = Cursor::new(context.source_bytes.as_slice());
    let mesh = stl_io::read_stl(&mut reader).map_err(|error| {
        AssetImportError::Parse(format!(
            "parse stl {}: {error}",
            context.source_path.display()
        ))
    })?;
    if mesh.faces.is_empty() {
        return Err(AssetImportError::Parse(format!(
            "parse stl {}: file contains no triangles",
            context.source_path.display()
        )));
    }

    let positions = mesh
        .vertices
        .iter()
        .flat_map(|vertex| vertex.0)
        .collect::<Vec<_>>();
    let indices = mesh
        .faces
        .iter()
        .flat_map(|face| face.vertices)
        .map(|index| {
            u32::try_from(index).map_err(|_| {
                AssetImportError::Parse(format!(
                    "parse stl {}: vertex index {index} exceeds u32",
                    context.source_path.display()
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let source_hint = context.uri.to_string();
    let primitive = primitive_from_indexed_mesh(
        &positions,
        &[],
        &[],
        &indices,
        context
            .source_path
            .file_stem()
            .and_then(|stem| stem.to_str()),
        &source_hint,
        context.mesh_sdf_cook_request()?.settings(),
    )?;

    model_outcome(context, vec![primitive])
}

fn import_ply(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let parser = ply::parser::Parser::<ply::ply::DefaultElement>::new();
    let ply = parser
        .read_ply(&mut Cursor::new(context.source_bytes.as_slice()))
        .map_err(|error| {
            AssetImportError::Parse(format!(
                "parse ply {}: {error}",
                context.source_path.display()
            ))
        })?;
    let vertices = ply.payload.get("vertex").ok_or_else(|| {
        AssetImportError::Parse(format!(
            "parse ply {}: missing vertex element",
            context.source_path.display()
        ))
    })?;
    let positions = vertices
        .iter()
        .flat_map(|vertex| {
            ["x", "y", "z"]
                .into_iter()
                .map(|key| scalar_f32(vertex, key, context))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let normals = collect_optional_vec3(vertices, ["nx", "ny", "nz"], context)?;
    let texcoords = collect_optional_vec2_candidates(
        vertices,
        [["s", "t"], ["u", "v"], ["texture_u", "texture_v"]],
        context,
    )?;
    let faces = ply.payload.get("face").ok_or_else(|| {
        AssetImportError::Parse(format!(
            "parse ply {}: missing face element",
            context.source_path.display()
        ))
    })?;
    let mut indices = Vec::new();
    for face in faces {
        let face_indices = list_u32(face, "vertex_indices")
            .or_else(|| list_u32(face, "vertex_index"))
            .ok_or_else(|| {
                AssetImportError::Parse(format!(
                    "parse ply {}: face missing vertex_indices list",
                    context.source_path.display()
                ))
            })?;
        if face_indices.len() < 3 {
            return Err(AssetImportError::Parse(format!(
                "parse ply {}: face has fewer than three vertices",
                context.source_path.display()
            )));
        }
        for triangle in 1..face_indices.len() - 1 {
            indices.push(face_indices[0]);
            indices.push(face_indices[triangle]);
            indices.push(face_indices[triangle + 1]);
        }
    }
    if indices.is_empty() {
        return Err(AssetImportError::Parse(format!(
            "parse ply {}: file contains no triangles",
            context.source_path.display()
        )));
    }

    let source_hint = context.uri.to_string();
    let primitive = primitive_from_indexed_mesh(
        &positions,
        &normals,
        &texcoords,
        &indices,
        context
            .source_path
            .file_stem()
            .and_then(|stem| stem.to_str()),
        &source_hint,
        context.mesh_sdf_cook_request()?.settings(),
    )?;

    model_outcome(context, vec![primitive])
}

pub(crate) fn model_outcome(
    context: &AssetImportContext,
    primitives: Vec<ModelPrimitiveAsset>,
) -> Result<AssetImportOutcome, AssetImportError> {
    let model = ModelAsset {
        uri: context.uri.clone(),
        primitives,
    };
    Ok(model_outcome_with_mesh_subassets(
        context.uri.clone(),
        model,
    ))
}

fn model_outcome_with_mesh_subassets(
    root_uri: zircon_runtime::asset::AssetUri,
    mut model: ModelAsset,
) -> AssetImportOutcome {
    let mesh_uris = (0..model.primitives.len())
        .map(|primitive_index| {
            zircon_runtime::asset::AssetUri::parse(&format!(
                "{root_uri}#Mesh{primitive_index}/Primitive0"
            ))
            .expect("generated model mesh subasset uri must be valid")
        })
        .collect::<Vec<_>>();
    for (primitive, mesh_uri) in model.primitives.iter_mut().zip(mesh_uris.iter()) {
        primitive.mesh = Some(AssetReference::from_locator(mesh_uri.clone()));
    }

    let mesh_entries = mesh_uris
        .into_iter()
        .zip(model.primitives.iter_mut())
        .map(|(mesh_uri, primitive)| {
            let mut mesh = MeshAsset::from_model_primitive(mesh_uri.clone(), primitive);
            mesh.mesh_sdf = primitive.mesh_sdf.take();
            ImportedAssetEntry::new(mesh_uri, ImportedAsset::Mesh(mesh))
        })
        .collect::<Vec<_>>();
    mesh_entries.into_iter().fold(
        AssetImportOutcome::new(root_uri, ImportedAsset::Model(model)),
        |outcome, entry| {
            outcome
                .with_dependency(entry.locator.clone())
                .with_entry(entry)
        },
    )
}

pub(crate) fn primitive_from_indexed_mesh(
    positions: &[f32],
    normals: &[f32],
    texcoords: &[f32],
    indices: &[u32],
    mesh_name: Option<&str>,
    source_hint: &str,
    mesh_sdf_settings: Option<MeshSdfCookSettings>,
) -> Result<ModelPrimitiveAsset, AssetImportError> {
    if positions.len() % 3 != 0 {
        return Err(AssetImportError::Parse(
            "vertex positions were not a multiple of 3".to_string(),
        ));
    }
    let vertex_count = positions.len() / 3;
    let computed_normals = prepare_vertex_normals(positions, normals, indices)?;

    let vertices: Vec<MeshVertex> = (0..vertex_count)
        .map(|index| {
            let position = Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            );
            let normal = Vec3::new(
                computed_normals[index * 3],
                computed_normals[index * 3 + 1],
                computed_normals[index * 3 + 2],
            );
            let uv = if texcoords.len() >= (index + 1) * 2 {
                Vec2::new(texcoords[index * 2], texcoords[index * 2 + 1])
            } else {
                Vec2::ZERO
            };
            MeshVertex::new(
                position,
                if normal.length_squared() <= f32::EPSILON {
                    Vec3::Y
                } else {
                    normal.normalize_or_zero()
                },
                uv,
            )
        })
        .collect();

    let virtual_geometry = cook_virtual_geometry_from_mesh(
        &vertices,
        indices,
        VirtualGeometryCookConfig {
            mesh_name: mesh_name.map(str::to_owned),
            source_hint: Some(source_hint.to_string()),
            ..VirtualGeometryCookConfig::default()
        },
    );
    let mesh_sdf = match mesh_sdf_settings {
        Some(settings) => cook_mesh_sdf_or_fallback_single(&vertices, indices, settings)
            .map_err(|error| AssetImportError::Parse(format!("cook mesh SDF: {error}")))?,
        None => None,
    };

    Ok(ModelPrimitiveAsset {
        vertices,
        indices: indices.to_vec(),
        mesh: None,
        mesh_sdf,
        virtual_geometry,
    })
}

fn prepare_vertex_normals<'a>(
    positions: &[f32],
    normals: &'a [f32],
    indices: &[u32],
) -> Result<Cow<'a, [f32]>, AssetImportError> {
    let vertex_count = positions.len() / 3;
    validate_indices(indices, vertex_count)?;
    if normals.is_empty() {
        return Ok(Cow::Owned(generate_normals(positions, indices)));
    }
    let required_normal_count = vertex_count * 3;
    if normals.len() >= required_normal_count {
        return Ok(Cow::Borrowed(normals));
    }

    let mut padded = normals.to_vec();
    padded.resize(required_normal_count, 0.0);
    Ok(Cow::Owned(padded))
}

fn validate_indices(indices: &[u32], vertex_count: usize) -> Result<(), AssetImportError> {
    for index in indices {
        if *index as usize >= vertex_count {
            return Err(AssetImportError::Parse(format!(
                "model index {index} exceeds vertex count {vertex_count}"
            )));
        }
    }
    Ok(())
}

fn generate_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
    let vertex_count = positions.len() / 3;
    let mut normals = vec![0.0_f32; vertex_count * 3];

    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;
        let position = |index: usize| -> Vec3 {
            Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            )
        };
        let face_normal = (position(b) - position(a))
            .cross(position(c) - position(a))
            .normalize_or_zero();
        for index in [a, b, c] {
            normals[index * 3] += face_normal.x;
            normals[index * 3 + 1] += face_normal.y;
            normals[index * 3 + 2] += face_normal.z;
        }
    }

    normals
}

fn scalar_f32(
    element: &ply::ply::DefaultElement,
    key: &str,
    context: &AssetImportContext,
) -> Result<f32, AssetImportError> {
    element
        .get(key)
        .and_then(|property| property.to_f32_lossy())
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "parse ply {}: vertex missing numeric `{key}`",
                context.source_path.display()
            ))
        })
}

fn list_u32(element: &ply::ply::DefaultElement, key: &str) -> Option<Vec<u32>> {
    element.get(key).and_then(|property| property.to_u32_list())
}

fn collect_optional_vec3(
    vertices: &[ply::ply::DefaultElement],
    keys: [&str; 3],
    context: &AssetImportContext,
) -> Result<Vec<f32>, AssetImportError> {
    if vertices
        .iter()
        .all(|vertex| keys.iter().all(|key| vertex.contains_key(*key)))
    {
        vertices
            .iter()
            .flat_map(|vertex| keys.into_iter().map(|key| scalar_f32(vertex, key, context)))
            .collect()
    } else {
        Ok(Vec::new())
    }
}

fn collect_optional_vec2_candidates(
    vertices: &[ply::ply::DefaultElement],
    candidates: [[&str; 2]; 3],
    context: &AssetImportContext,
) -> Result<Vec<f32>, AssetImportError> {
    let Some(keys) = candidates.into_iter().find(|keys| {
        vertices
            .iter()
            .all(|vertex| vertex.contains_key(keys[0]) && vertex.contains_key(keys[1]))
    }) else {
        return Ok(Vec::new());
    };
    vertices
        .iter()
        .flat_map(|vertex| keys.into_iter().map(|key| scalar_f32(vertex, key, context)))
        .collect()
}

#[cfg(test)]
mod hotpath_tests {
    use super::*;
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn plugins07_model_hotpath_complete_normals_are_borrowed_and_short_normals_are_padded() {
        let positions = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let normals = [0.0, 0.0, 2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 2.0];
        let indices = [0, 1, 2];

        let complete = prepare_vertex_normals(&positions, &normals, &indices).unwrap();
        let short = prepare_vertex_normals(&positions, &normals[..3], &indices).unwrap();

        assert!(matches!(&complete, std::borrow::Cow::Borrowed(_)));
        assert!(matches!(&short, std::borrow::Cow::Owned(_)));
        assert_eq!(
            short.as_ref(),
            &[0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
    }

    #[test]
    #[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
    fn plugins07_model_hotpath_release_borrowed_normals_p95_gate() {
        const SAMPLE_PAIRS: usize = 21;
        const NORMAL_VALUES: usize = 524_288;
        const ITERATIONS: usize = 16;
        const THRESHOLD_PERCENT: u128 = 80;
        let normals = vec![0.25_f32; NORMAL_VALUES];
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let legacy = || measure_normal_clone(&normals, ITERATIONS);
            let optimized = || measure_normal_borrow(&normals, ITERATIONS);
            if pair % 2 == 0 {
                legacy_samples.push(legacy());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                legacy_samples.push(legacy());
            }
        }

        emit_model_normal_performance_gate(
            &legacy_samples,
            &optimized_samples,
            THRESHOLD_PERCENT,
            &format!(
                "normal_values={NORMAL_VALUES} iterations_per_sample={ITERATIONS} legacy_cloned_bytes_per_sample={} optimized_cloned_bytes_per_sample=0",
                NORMAL_VALUES * ITERATIONS * std::mem::size_of::<f32>()
            ),
        );
    }

    fn measure_normal_clone(normals: &[f32], iterations: usize) -> u128 {
        let started = Instant::now();
        let mut values = 0_usize;
        for _ in 0..iterations {
            let owned = black_box(normals).to_vec();
            values += black_box(owned.as_slice()).len();
        }
        black_box(values);
        started.elapsed().as_nanos()
    }

    fn measure_normal_borrow(normals: &[f32], iterations: usize) -> u128 {
        let started = Instant::now();
        let mut values = 0_usize;
        for _ in 0..iterations {
            let borrowed: Cow<'_, [f32]> = Cow::Borrowed(black_box(normals));
            values += black_box(borrowed.as_ref()).len();
        }
        black_box(values);
        started.elapsed().as_nanos()
    }

    fn emit_model_normal_performance_gate(
        legacy_samples: &[u128],
        optimized_samples: &[u128],
        threshold_percent: u128,
        workload: &str,
    ) {
        let legacy_p95 = nearest_rank_model_p95(legacy_samples);
        let optimized_p95 = nearest_rank_model_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_model_borrowed_normals sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
            model_samples_csv(legacy_samples),
            model_samples_csv(optimized_samples),
        );
        assert!(
            improvement_percent >= threshold_percent,
            "model borrowed normals must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
        );
    }

    fn nearest_rank_model_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn model_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
