use std::sync::Arc;

use crate::asset::ModelAsset;
use crate::core::resource::ResourceId;

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn load_model_asset(&self, id: ResourceId) -> Option<Arc<ModelAsset>> {
        let asset_manager = self.asset_manager().ok()?;
        load_model_asset_with_cache(
            self.models
                .get(&id)
                .map(|prepared| (&prepared.asset, prepared.source_revision)),
            self.resource_revision(id).ok(),
            || asset_manager.load_model_asset(id).ok().map(Arc::new),
        )
    }
}

fn load_model_asset_with_cache<F>(
    prepared: Option<(&Arc<ModelAsset>, u64)>,
    current_revision: Option<u64>,
    fallback_load: F,
) -> Option<Arc<ModelAsset>>
where
    F: FnOnce() -> Option<Arc<ModelAsset>>,
{
    if let (Some((asset, prepared_revision)), Some(current_revision)) = (prepared, current_revision)
    {
        if prepared_revision == current_revision {
            return Some(Arc::clone(asset));
        }
    }
    fallback_load()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::load_model_asset_with_cache;
    use crate::asset::{
        AssetUri, MeshVertex, ModelAsset, ModelPrimitiveAsset, VirtualGeometryAsset,
    };
    use crate::core::math::{Vec2, Vec3};

    const ENSURE_MODEL_SOURCE: &str = include_str!("resource_streamer_ensure_model.rs");

    #[test]
    fn ensure_model_consumes_the_shared_asset_without_rewrapping_it() {
        let body = ENSURE_MODEL_SOURCE
            .split("pub(crate) fn ensure_model(")
            .nth(1)
            .expect("model preparation entry must remain present");
        let compact = body
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();

        assert!(compact.contains("self.load_model_asset(id)"));
        assert_eq!(compact.matches(".load_model_asset(id)").count(), 1);
        assert!(!compact.contains("Arc::new(asset_manager"));
    }

    #[test]
    fn current_prepared_model_asset_short_circuits_fallback_loading() {
        let cached = Arc::new(cooked_model_asset("res://models/cached.model.toml"));

        let loaded = load_model_asset_with_cache(Some((&cached, 7)), Some(7), || {
            panic!("current prepared asset should short-circuit fallback loading")
        });

        assert!(
            Arc::ptr_eq(
                loaded.as_ref().expect("current prepared asset must load"),
                &cached,
            ),
            "current prepared model asset must be borrowed through the existing Arc"
        );
        let loaded_again = load_model_asset_with_cache(Some((&cached, 7)), Some(7), || {
            panic!("repeated current prepared hit should not invoke fallback loading")
        });
        assert!(Arc::ptr_eq(
            loaded_again
                .as_ref()
                .expect("repeated current prepared asset must load"),
            &cached,
        ));
    }

    #[test]
    fn composite_geometry_revision_does_not_invalidate_source_asset_cache() {
        let cached = Arc::new(cooked_model_asset("res://models/external-mesh.model.toml"));
        let composite_geometry_revision = 0xfeed_beef_u64;
        let source_revision = 7;
        assert_ne!(composite_geometry_revision, source_revision);

        let loaded = load_model_asset_with_cache(
            Some((&cached, source_revision)),
            Some(source_revision),
            || panic!("composite geometry revision must not force source asset reload"),
        );

        assert!(Arc::ptr_eq(
            loaded.as_ref().expect("current prepared asset must load"),
            &cached,
        ));
    }

    #[test]
    fn stale_prepared_model_asset_falls_back_to_latest_asset_load() {
        let cached = Arc::new(cooked_model_asset("res://models/cached.model.toml"));
        let fresh = plain_model_asset("res://models/fresh.model.toml");

        let loaded = load_model_asset_with_cache(Some((&cached, 7)), Some(8), || {
            Some(Arc::new(fresh.clone()))
        });

        let loaded = loaded.expect("stale model asset must use fallback");
        assert!(!Arc::ptr_eq(&loaded, &cached));
        assert_eq!(loaded.as_ref(), &fresh);
    }

    #[test]
    #[ignore = "current-source realistic-payload pointer-reuse profile"]
    fn shader06_current_model_asset_hits_reuse_realistic_payload_storage() {
        const INSTANCE_COUNT: usize = 16_384;
        const VERTEX_COUNT: usize = 262_144;
        const INDEX_COUNT: usize = 786_432;

        let mut model = plain_model_asset("res://models/current-source-profile.model.toml");
        model.primitives[0].vertices =
            vec![MeshVertex::new(Vec3::ZERO, Vec3::Y, Vec2::ZERO); VERTEX_COUNT];
        model.primitives[0].indices = (0..INDEX_COUNT)
            .map(|index| (index % VERTEX_COUNT) as u32)
            .collect();
        let cached = Arc::new(model);
        let vertex_pointer = cached.primitives[0].vertices.as_ptr();
        let index_pointer = cached.primitives[0].indices.as_ptr();
        let payload_bytes = std::mem::size_of_val(cached.primitives[0].vertices.as_slice())
            + std::mem::size_of_val(cached.primitives[0].indices.as_slice());

        for _ in 0..INSTANCE_COUNT {
            let loaded = load_model_asset_with_cache(Some((&cached, 11)), Some(11), || {
                panic!("current prepared model must not deep-load the realistic payload")
            })
            .expect("current prepared model must remain available");
            assert!(Arc::ptr_eq(&loaded, &cached));
            assert_eq!(loaded.primitives[0].vertices.as_ptr(), vertex_pointer);
            assert_eq!(loaded.primitives[0].indices.as_ptr(), index_pointer);
        }

        println!(
            "PERF_RESULT shader06_model_asset_arc_reuse instances={INSTANCE_COUNT} vertices={VERTEX_COUNT} indices={INDEX_COUNT} payload_bytes={payload_bytes} pointer_reuses={INSTANCE_COUNT} fallback_loads=0"
        );
    }

    fn cooked_model_asset(uri: &str) -> ModelAsset {
        ModelAsset {
            uri: AssetUri::parse(uri).unwrap(),
            primitives: vec![ModelPrimitiveAsset {
                vertices: Vec::new(),
                indices: Vec::new(),
                mesh: None,
                mesh_sdf: None,
                virtual_geometry: Some(VirtualGeometryAsset {
                    root_page_table: vec![1],
                    ..VirtualGeometryAsset::default()
                }),
            }],
        }
    }

    fn plain_model_asset(uri: &str) -> ModelAsset {
        ModelAsset {
            uri: AssetUri::parse(uri).unwrap(),
            primitives: vec![ModelPrimitiveAsset {
                vertices: Vec::new(),
                indices: Vec::new(),
                mesh: None,
                mesh_sdf: None,
                virtual_geometry: None,
            }],
        }
    }
}
