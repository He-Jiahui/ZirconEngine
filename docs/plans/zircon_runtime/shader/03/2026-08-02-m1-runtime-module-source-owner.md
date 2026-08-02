---
Plan: docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md
Milestone: M1
Status: in_progress
Files: ["docs/plans/zircon_runtime/shader/03/2026-08-02-m1-runtime-module-source-owner.md", "zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs", "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_construction.rs", "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs", "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs", "zircon_runtime/src/graphics/scene/render_product_streamer_tests/readiness_diagnostics/shader_redirect.rs", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report/registrations.rs", "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report/tests.rs", "zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs", "zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs/tests.rs"]
---

# Shader03 M1 Runtime Module Source Owner

## Scope

The open Shader03 failure is a product-runtime ownership gap, not a registry or
prewarm-only gap. This manifest owns the current source-only project and plugin
module bindings that feed the existing `ShaderModuleRegistry` through the real
`ResourceStreamer` and template assembly path.

`ShaderModuleSourceBinding` keeps the authoritative owner identity, import path,
shared `Arc<str>` source body, content hash, and diagnostic origin. Project
source-only imports are prepared through declared dependency ids, while package
modules are supplied during runtime registration with package-relative path and
bounded-read rules. The renderer does not rescan project or package files while
assembling a material.

ResourceStreamer shares one per-traversal state across redirect and
dependency-id preparation edges. Its recursion-stack set stops malformed project
dependency cycles, while its completed set avoids reopening an already successful
shared subgraph through a second DAG edge. The later ShaderModuleRegistry still
reports the assembly-level cycle diagnostic from real WGSL module includes.

## Performance And Ownership Constraints

- Preserve one registry and the precedence order `redirect > project source-only > plugin`.
- Retain source bodies through shared `Arc<str>` bindings; do not create render-time
  package reads or a second project/plugin registry.
- Native package loading is bounded to 4 MiB per module, 64 modules per package,
  and 16 MiB total module source per package.

## Acceptance

- The focused source-only and plugin product tests assemble a real material
  through `ResourceStreamer` and `ShaderModuleRegistry`, then Naga-validate it;
  the plugin path also creates a WGPU shader module.
- The product source-only binding overrides the same-token plugin binding, and
  the final template records the selected body and content hash.
- Shared `dependency_ids` descendants prepare once per top-level request, and a
  two-node dependency plus WGSL-include cycle remains an actionable
  `CircularModuleInclude` diagnostic after product source preparation.
- Native missing-source diagnostics, linked registration, module-count and
  aggregate-source budgets, and cross-package token conflicts remain actionable.
- Managed Windows locked Rust validation runs against this manifest after the
  Runtime04 current-source compiler gate is green. This manifest is not accepted
  until that evidence exists and the linked failure is returned as `fixed-*`.

## Current Status

Implementation and focused regression coverage are complete, including the
current dependency-cycle guard. `cargo metadata --locked` currently resolves
the workspace, replacing the earlier lockfile parse-stage blocker. No managed
Rust/WGPU product result has yet been recorded for this manifest, so it remains
`in_progress` and the linked failure remains open.

The post-fix independent second review reports `Critical 0 / Important 0 /
Minor 0`: the traversal state separates active-cycle termination from
successful-subgraph deduplication, cleans up after failed preparation, and the
product source path preserves the exact `CircularModuleInclude` diagnostic.

The native-package follow-up closes the remaining feature-only package
projection gap. A package-level shader module now registers through every
active runtime feature report when the package has no primary runtime report,
covering both FeatureExtension packages and ordinary packages whose only
runtime entry is optional. A package with no runtime feature does not resolve
shader files. Packages with a primary runtime module retain the original
single ordinary-report source owner. Runtime module assembly collapses only
identical `(owner_id, import_path, content_hash)` copies produced by multiple
active features, so distinct package/token or content conflicts remain visible
to the existing single ResourceStreamer source map. Regression coverage proves
both native package-relative source forms reach the feature report and that
feature registration inputs retain it for graphics startup. Focused source
parsing and diff integrity pass. The post-fix independent second review reports
`Critical 0 / Important 0 / Minor 0`: active feature reports carry their
package sources, identical owner/import/hash copies collapse once, and distinct
package owners stay diagnosable by the ResourceStreamer conflict check. Managed
Windows Rust/WGPU product evidence remains the acceptance gate, so M1 remains
`in_progress`.
