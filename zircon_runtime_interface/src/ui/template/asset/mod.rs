mod action_policy;
mod binding;
mod compiler;
mod component_contract;
mod document;
mod invalidation;
mod localization;
mod prototype;
mod resource_ref;
mod schema;
mod style;

pub use action_policy::{
    UiActionHostPolicy, UiActionPolicyDiagnostic, UiActionPolicyDiagnosticSeverity,
    UiActionPolicyReport, UiActionSideEffectClass,
};
pub use binding::{
    UiActionPayloadFieldName, UiBindingContractTerm, UiBindingDiagnostic, UiBindingDiagnosticCode,
    UiBindingDiagnosticSeverity, UiBindingExpression, UiBindingExpressionEvaluationError,
    UiBindingExpressionParseError, UiBindingMissingValuePolicy, UiBindingMissingValueResolution,
    UiBindingReport, UiBindingSchemaNameError, UiBindingSchemaNameKind, UiBindingTarget,
    UiBindingTargetAssignment, UiBindingTargetKind, UiBindingTargetSchema,
    UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY, UI_BINDING_EXPRESSION_MAX_DEPTH,
    UI_BINDING_EXPRESSION_MAX_NODES, UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
    UI_BINDING_EXPRESSION_MAX_TOKENS, UI_BINDING_SCHEMA_NAME_MAX_BYTES,
};
pub use compiler::{
    UiBindingId, UiBindingPackageLifecycleStage, UiCompileCacheKey, UiCompiledActionId,
    UiCompiledActionPayloadField, UiCompiledActionPayloadValue, UiCompiledAssetArtifact,
    UiCompiledAssetCacheRecord, UiCompiledAssetDependency, UiCompiledAssetDependencyManifest,
    UiCompiledAssetHeader, UiCompiledAssetId, UiCompiledAssetPackageArtifactEntry,
    UiCompiledAssetPackageManifest, UiCompiledAssetPackageProfile, UiCompiledAssetPackageSection,
    UiCompiledAssetPackageValidationReport, UiCompiledBinding, UiCompiledBindingExpression,
    UiCompiledBindingGeneration, UiCompiledBindingHandle, UiCompiledBindingProgram,
    UiCompiledBindingTarget, UiCompiledBindingTargetEndpoint, UiCompiledBindingTargetId,
    UiCompiledBindingTargetKind, UiCompiledControlId, UiCompiledNodeBindings, UiCompiledNodeId,
    UiCompiledRouteId, UiPropertyId, UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION,
    UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION, UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION,
};
pub use component_contract::{
    UiComponentApiVersion, UiComponentBindingContract, UiComponentContractDiagnostic,
    UiComponentContractDiagnosticCode, UiComponentFocusContract, UiComponentPublicContract,
    UiPublicBindingRoute, UiPublicPart, UiRootClassPolicy,
};
pub use document::{
    parse_component_reference, UiActionRef, UiAssetDocument, UiAssetError, UiAssetHeader,
    UiAssetImports, UiAssetKind, UiChildMount, UiComponentDefinition, UiComponentParamSchema,
    UiNamedSlotSchema, UiNodeDefinition, UiNodeDefinitionKind, UiStyleDeclarationBlock,
    UiStyleRule, UiStyleScope, UiStyleSheet,
};
pub use invalidation::{
    UiAssetChange, UiAssetFingerprint, UiInvalidationDiagnostic, UiInvalidationDiagnosticSeverity,
    UiInvalidationImpact, UiInvalidationReport, UiInvalidationSnapshot, UiInvalidationStage,
};
pub use localization::{
    UiLocalizationDependency, UiLocalizationDiagnostic, UiLocalizationDiagnosticSeverity,
    UiLocalizationReport, UiLocalizationTextCandidate, UiLocalizedTextRef, UiTextDirection,
};
pub use prototype::{
    UiComponentPrototype, UiDocumentPrototype, UiNodePrototype, UiPrototypeChildMount,
    UiPrototypeNodeHandle, UiRawAssetPrototype, UiStylePrototype,
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
pub use style::{
    UiSelector, UiSelectorCombinator, UiSelectorSegment, UiSelectorSpecificity, UiSelectorToken,
};
