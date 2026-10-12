#[path = "../../support/types/dependency_signature.rs"]
mod dependency_signature;
#[path = "../../support/types/module_signature.rs"]
mod module_signature;
#[path = "../../support/types/pending_manifest.rs"]
mod pending_manifest;
#[path = "types/static_manifest.rs"]
mod static_manifest;

pub(super) use self::dependency_signature::OptionalFeatureDependencySignature;
pub(super) use self::module_signature::OptionalFeatureModuleSignature;
pub(super) use self::pending_manifest::PendingOptionalFeatureManifest;
pub(in crate::tests::optional_feature_manifest) use self::static_manifest::StaticOptionalFeatureManifest;
