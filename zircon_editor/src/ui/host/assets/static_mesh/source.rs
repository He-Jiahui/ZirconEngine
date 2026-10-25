use zircon_runtime::asset::assets::{ModelAssetOverview, SceneMeshInstanceAsset};

/// Source facts for an already selected model, not static/skinned type admission.
///
/// ModelAsset does not retain the original node/skin identity. The provider must supply that
/// identity, source revision and same-publication scope before routing or enabling an editor.
/// Neither unweighted vertices nor a Mesh resource kind proves a static asset. This projection
/// creates no preview session, registry entry, document, history, geometry copy or write command.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticMeshSource {
    /// Original Runtime overview; source ordinals apply only to this captured model.
    /// Statistics are not geometry validation, GPU readiness or stable asset identities.
    pub geometry: ModelAssetOverview,
    pub instance: StaticMeshInstanceObservation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StaticMeshInstanceObservation {
    /// No actual instance context was supplied; this never means zero material bindings.
    Unknown,
    /// Exact base-model primitive mesh references matched. All original fields, including
    /// per-primitive materials, fallback material, LODs and morph weights, remain intact.
    /// References are observations; registry resolution/navigation/readiness is still separate.
    Known(Box<SceneMeshInstanceAsset>),
}
