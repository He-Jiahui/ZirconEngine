use crate::asset::ModelAsset;
use crate::core::resource::ResourceId;

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn load_model_asset(&self, id: ResourceId) -> Option<ModelAsset> {
        load_model_asset_with_cache(
            self.models
                .get(&id)
                .map(|prepared| (prepared.asset.as_ref(), prepared.revision)),
            self.resource_revision(id).ok(),
            || self.asset_manager.load_model_asset(id).ok(),
        )
    }
}

fn load_model_asset_with_cache<F>(
    prepared: Option<(&ModelAsset, u64)>,
    current_revision: Option<u64>,
    fallback_load: F,
) -> Option<ModelAsset>
where
    F: FnOnce() -> Option<ModelAsset>,
{
    if let (Some((asset, prepared_revision)), Some(current_revision)) = (prepared, current_revision)
    {
        if prepared_revision == current_revision {
            return Some(asset.clone());
        }
    }
    fallback_load()
}

#[cfg(test)]
mod tests {
    use super::load_model_asset_with_cache;
    use crate::asset::{AssetUri, ModelAsset, ModelPrimitiveAsset, VirtualGeometryAsset};

    #[test]
    fn current_prepared_model_asset_short_circuits_fallback_loading() {
        let cached = cooked_model_asset("res://models/cached.model.toml");

        let loaded = load_model_asset_with_cache(Some((&cached, 7)), Some(7), || {
            panic!("current prepared asset should short-circuit fallback loading")
        });

        assert_eq!(loaded, Some(cached));
    }

    #[test]
    fn stale_prepared_model_asset_falls_back_to_latest_asset_load() {
        let cached = cooked_model_asset("res://models/cached.model.toml");
        let fresh = plain_model_asset("res://models/fresh.model.toml");

        let loaded =
            load_model_asset_with_cache(Some((&cached, 7)), Some(8), || Some(fresh.clone()));

        assert_eq!(loaded, Some(fresh));
    }

    fn cooked_model_asset(uri: &str) -> ModelAsset {
        ModelAsset {
            uri: AssetUri::parse(uri).unwrap(),
            primitives: vec![ModelPrimitiveAsset {
                vertices: Vec::new(),
                indices: Vec::new(),
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
                virtual_geometry: None,
            }],
        }
    }
}
