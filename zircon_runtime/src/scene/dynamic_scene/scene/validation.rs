use std::collections::HashSet;

use crate::core::framework::scene::ComponentTypeDescriptor;

use super::DynamicScene;
use crate::scene::dynamic_scene::DynamicSceneError;
use zircon_runtime_interface::serialization::VersionedSchema;

pub(super) fn ensure_scene_supported(scene: &DynamicScene) -> Result<(), DynamicSceneError> {
    validate_format_version(scene)?;
    ensure_component_type_descriptors(scene)?;
    ensure_unique_sources(scene)
}

fn validate_format_version(scene: &DynamicScene) -> Result<(), DynamicSceneError> {
    if scene.payload_header.schema_id != DynamicScene::SCHEMA {
        return Err(DynamicSceneError::UnsupportedSchema {
            expected: DynamicScene::SCHEMA.as_str().to_string(),
            actual: scene.payload_header.schema_id.as_str().to_string(),
        });
    }
    if scene.payload_header.schema_version != DynamicScene::VERSION {
        return Err(DynamicSceneError::UnsupportedFormatVersion {
            expected: DynamicScene::VERSION,
            actual: scene.payload_header.schema_version,
        });
    }
    Ok(())
}

fn ensure_unique_sources(scene: &DynamicScene) -> Result<(), DynamicSceneError> {
    let mut seen = HashSet::new();
    for entity in &scene.entities {
        if !seen.insert(entity.source_entity) {
            return Err(DynamicSceneError::DuplicateSourceEntity {
                entity: entity.source_entity,
            });
        }
    }
    Ok(())
}

fn ensure_component_type_descriptors(scene: &DynamicScene) -> Result<(), DynamicSceneError> {
    let mut seen = HashSet::new();
    for descriptor in &scene.component_types {
        if !seen.insert(descriptor.type_id.as_str()) {
            return Err(DynamicSceneError::DuplicateComponentTypeDescriptor {
                type_id: descriptor.type_id.clone(),
            });
        }
        validate_component_type_descriptor(descriptor)?;
    }
    Ok(())
}

fn validate_component_type_descriptor(
    descriptor: &ComponentTypeDescriptor,
) -> Result<(), DynamicSceneError> {
    if !component_type_belongs_to_plugin(&descriptor.type_id, &descriptor.plugin_id) {
        return Err(DynamicSceneError::InvalidComponentTypeDescriptor {
            type_id: descriptor.type_id.clone(),
            reason: format!(
                "component type must be prefixed by plugin id `{}`",
                descriptor.plugin_id
            ),
        });
    }
    crate::scene::reflect::registration_from_component_descriptor(descriptor).map_err(|error| {
        DynamicSceneError::InvalidComponentTypeDescriptor {
            type_id: descriptor.type_id.clone(),
            reason: error.to_string(),
        }
    })?;
    Ok(())
}

fn component_type_belongs_to_plugin(type_id: &str, plugin_id: &str) -> bool {
    let Some(suffix) = type_id.strip_prefix(plugin_id) else {
        return false;
    };
    suffix.starts_with('.')
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::scene::dynamic_scene::DynamicEntity;
    use crate::scene::{NodeKind, World};

    use super::*;

    const ID_ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_ID_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn validation_ids() -> Vec<(u64, String)> {
        (0..ID_ADMISSION_COUNT)
            .map(|index| {
                let identity = (index * 4_099) % UNIQUE_ID_COUNT;
                (
                    identity as u64,
                    format!("plugin.generated.component.with.long.identity.{identity:05}"),
                )
            })
            .collect()
    }

    fn ordered_unique_counts(ids: &[(u64, String)]) -> (usize, usize) {
        let mut sources = BTreeSet::new();
        let mut component_types = BTreeSet::new();
        let source_count = ids
            .iter()
            .filter(|(source, _)| sources.insert(*source))
            .count();
        let component_count = ids
            .iter()
            .filter(|(_, type_id)| component_types.insert(type_id.as_str()))
            .count();
        (source_count, component_count)
    }

    fn hash_unique_counts(ids: &[(u64, String)]) -> (usize, usize) {
        let mut sources = HashSet::new();
        let mut component_types = HashSet::new();
        let source_count = ids
            .iter()
            .filter(|(source, _)| sources.insert(*source))
            .count();
        let component_count = ids
            .iter()
            .filter(|(_, type_id)| component_types.insert(type_id.as_str()))
            .count();
        (source_count, component_count)
    }

    fn dynamic_entity(source_entity: u64) -> DynamicEntity {
        let mut world = World::empty();
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("fixture node should spawn");
        DynamicEntity::new(
            source_entity,
            world.node_record(entity).expect("fixture node record"),
            Vec::new(),
        )
    }

    #[test]
    fn runtime_hash_recovery_batch_runtime52_hash_validation_preserves_first_duplicate_source() {
        let mut scene = DynamicScene::empty();
        scene.entities = vec![dynamic_entity(41), dynamic_entity(7), dynamic_entity(41)];

        assert!(matches!(
            scene.ensure_supported(),
            Err(DynamicSceneError::DuplicateSourceEntity { entity: 41 })
        ));
    }

    #[test]
    fn runtime_hash_recovery_batch_runtime52_scene_validation_uses_hash_membership() {
        let source = include_str!("validation.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert_eq!(production.matches("HashSet::new()").count(), 2);
        assert!(production.contains("seen.insert(entity.source_entity)"));
        assert!(production.contains("seen.insert(descriptor.type_id.as_str())"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn runtime_hash_recovery_batch_runtime52_dynamic_scene_hash_validation_performance_evidence() {
        let ids = validation_ids();
        assert_eq!(ordered_unique_counts(&ids), hash_unique_counts(&ids));

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_unique_counts(black_box(&ids)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_unique_counts(black_box(&ids)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_unique_counts(black_box(&ids)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_unique_counts(black_box(&ids)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "RUNTIME52_DYNAMIC_SCENE_HASH_VALIDATION_BENCH_V1 \
             admissions={ID_ADMISSION_COUNT} unique_ids={UNIQUE_ID_COUNT} \
             borrowed_component_type_identity=true ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-validation P95 {:?} exceeded 60% of ordered-validation P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }

    #[test]
    fn component_descriptor_uniqueness_borrows_type_ids() {
        let source = include_str!("validation.rs");
        let validation = source
            .split("fn ensure_component_type_descriptors")
            .nth(1)
            .and_then(|source| source.split("fn validate_component_type_descriptor").next())
            .expect("read component descriptor validation body");

        assert!(validation.contains("seen.insert(descriptor.type_id.as_str())"));
        assert!(
            !validation.contains("seen.insert(descriptor.type_id.clone())"),
            "descriptor uniqueness validation must not clone type-id strings"
        );
    }
}
