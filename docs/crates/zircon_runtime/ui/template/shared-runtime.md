---
related_code:
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/asset/loader.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/template/build/surface_builder.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/surface/surface.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/compiler/compile.rs
tests:
  - zircon_runtime/src/ui/tests/template_pipeline.rs
doc_type: module-detail
---

# Shared compiled templates and per-surface runtime state

This document describes the current production owners. It is a new source-backed
boundary document; it does not reconstruct the missing historical shared-template
document. The [compiler pipeline](pipeline.md) describes compilation stages and
binding contracts. This document describes what callers can reuse and what each
constructed surface owns.

## Reusable compilation inputs and products

`UiAssetLoader` reads author text or files and applies schema migration. Loading
does not create a running surface or resolve the host's imports and resources.
The host registers widget/style document snapshots on `UiDocumentCompiler` and
selects its component descriptor registry. The default compiler borrows the
shared registry; callers can supply an owned registry or a static shared registry.

The compiler produces `UiCompiledDocument`: an asset header, an expanded
`UiTemplateInstance`, resource dependencies, and diagnostics. Compilation does
not load those resources or create the runtime tree. Callers that consume
`into_template_instance` must preserve the resource report separately if needed.

`UiTemplateInstance` contains the expanded root and its corresponding compiled
binding program. It is reusable input to surface construction. A manually created
`UiTemplateInstance::new` has an empty compiled binding program. Changing its
public root does not automatically recompile that program; a compiled root and
binding program must remain paired.

`UiRuntimeCompiledAssetArtifact` stores a validation report and an instance.
Its versioned TOML envelope is data, not generated Rust source. Restoring the
artifact validates its envelope and binding-program structure; it does not load
resource dependencies, execute bindings, or install a surface. Existing content
identity limitations remain the responsibility of the artifact validation owner.

## Independent runtime construction and input

`UiTemplateSurfaceBuilder::build_surface_from_compiled_document` borrows the
compiled document and delegates to `build_surface`. Each invocation calls
`UiTemplateTreeBuilder::build_tree` with the supplied `UiTreeId`, creates a new
`UiSurface`, and installs a clone of the matching binding program. The tree
builder allocates runtime nodes and checks duplicate control IDs for that build.
The compiled document does not become a shared mutable runtime surface.

`UiSurface` owns its tree, focus, input and retained interaction state. Building
two surfaces from the same compiled document creates separate surface state;
the host must route input and updates to the intended surface identity.
Construction does not compute geometry or publish a rendered frame. Layout,
input dispatch, rendering and host resource admission remain their respective
runtime owners' work after construction.

## Source contracts and validation status

The existing `asset_compiler_is_the_single_template_compile_authority` test
constructs a surface through the production compiler and builder.
`legacy_recursive_template_document_is_not_a_runtime_compile_input` and
`compiled_template_artifact_stays_toml_envelope_leaf_dto_not_generated_source`
retain the compiler-authority and artifact boundaries.
These names identify existing test owners; this documentation update does not
claim a new execution result. Status: `source_applied`; `managed_tests_pending`.
