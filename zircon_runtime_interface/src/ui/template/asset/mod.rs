mod action_policy;
mod binding;
mod compiler;
mod component_contract;
mod document;
mod invalidation;
mod localization;
mod resource_ref;
mod schema;
mod style;

pub use action_policy::{
    UiActionHostPolicy, UiActionPolicyDiagnostic, UiActionPolicyDiagnosticSeverity,
    UiActionPolicyReport, UiActionSideEffectClass,
};
pub use binding::{
    UiBindingDiagnostic, UiBindingDiagnosticCode, UiBindingDiagnosticSeverity, UiBindingExpression,
    UiBindingExpressionParseError, UiBindingReport, UiBindingTarget, UiBindingTargetAssignment,
    UiBindingTargetKind, UiBindingTargetSchema,
};
pub use compiler::{
    UiCompileCacheKey, UiCompiledAssetArtifact, UiCompiledAssetCacheRecord,
    UiCompiledAssetDependency, UiCompiledAssetDependencyManifest, UiCompiledAssetHeader,
    UiCompiledAssetPackageArtifactEntry, UiCompiledAssetPackageManifest,
    UiCompiledAssetPackageProfile, UiCompiledAssetPackageSection,
    UiCompiledAssetPackageValidationReport, UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION,
    UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION, UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION,
};
pub use component_contract::{
    UiComponentApiVersion, UiComponentBindingContract, UiComponentContractDiagnostic,
    UiComponentContractDiagnosticCode, UiComponentFocusContract, UiComponentPublicContract,
    UiPublicBindingRoute, UiPublicPart, UiRootClassPolicy,
};
pub use document::{
    UiActionRef, UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetImports, UiAssetKind,
    UiChildMount, UiComponentDefinition, UiComponentParamSchema, UiNamedSlotSchema,
    UiNodeDefinition, UiNodeDefinitionKind, UiStyleDeclarationBlock, UiStyleRule, UiStyleScope,
    UiStyleSheet,
};
pub use invalidation::{
    UiAssetChange, UiAssetFingerprint, UiInvalidationDiagnostic, UiInvalidationDiagnosticSeverity,
    UiInvalidationImpact, UiInvalidationReport, UiInvalidationSnapshot, UiInvalidationStage,
};
pub use localization::{
    UiLocalizationDependency, UiLocalizationDiagnostic, UiLocalizationDiagnosticSeverity,
    UiLocalizationReport, UiLocalizationTextCandidate, UiLocalizedTextRef, UiTextDirection,
};
pub use resource_ref::{
    UiResourceCollectionReport, UiResourceDependency, UiResourceDependencySource,
    UiResourceDiagnostic, UiResourceDiagnosticSeverity, UiResourceFallbackMode,
    UiResourceFallbackPolicy, UiResourceKind, UiResourceRef,
};
pub use schema::{
    UiAssetMigrationOutcome, UiAssetMigrationReport, UiAssetMigrationStep, UiAssetSchemaDiagnostic,
    UiAssetSchemaDiagnosticSeverity, UiAssetSchemaSourceKind, UiAssetSchemaVersionPolicy,
    UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION, UI_ASSET_MINIMUM_SUPPORTED_SOURCE_SCHEMA_VERSION,
};
pub use style::{UiSelector, UiSelectorCombinator, UiSelectorToken};
