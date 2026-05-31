# Zircon Plugins Workspace

`zircon_plugins` is intentionally separate from the root workspace. Runtime/editor export and plugin CI can compile this workspace when plugin packages are needed, while the root workspace keeps the minimal runtime/editor core fast by default.

Runtime-backed plugin packages use this shape:

- `zircon_plugins/<plugin_id>/plugin.toml`
- `zircon_plugins/<plugin_id>/runtime/Cargo.toml`
- `zircon_plugins/<plugin_id>/editor/Cargo.toml`

Editor-only plugin packages omit the `runtime` crate:

- `zircon_plugins/<plugin_id>/plugin.toml`
- `zircon_plugins/<plugin_id>/editor/Cargo.toml`

Each `plugin.toml` is a serialized `PluginPackageManifest`: it should declare
the package id, semantic `version`, semantic `sdk_api_version`, category,
supported runtime targets and export platforms, package capabilities,
asset/content roots, module entries for runtime/editor/native/VM code,
dependencies/options/events/importers, and the default packaging strategies the
export planner may choose from. The root runtime manifest regression scans every
static `plugin.toml` and requires top-level `id`, `version`, `sdk_api_version`,
`display_name`, `category`, `description`, `maturity`, `supported_targets`, and
`capabilities`, plus non-empty `default_packaging` entries drawn from the known
export strategies (`source_template`, `library_embed`, `native_dynamic`), so
public package metadata and export packaging policy cannot silently fall back to
parser defaults. Public-facing package strings (`display_name`, `description`,
optional-feature `display_name`, and capability status `note`) must be trimmed
before plugin-window, readiness-report, runtime-report, or export diagnostics
consume them. Unknown top-level fields and unknown nested row fields are rejected
so misspelled static metadata cannot be silently ignored by parser defaults.
Static package ids must use lowercase snake-style tokens and match the package
directory name in `zircon_plugins/<plugin_id>/plugin.toml`.
Static `version` and `sdk_api_version` values must use `MAJOR.MINOR.PATCH`
numeric form with no leading zeroes. Static package ids must be globally unique,
and optional-feature ids must not collide with any package or feature identity.
Optional-feature ids must also use trimmed lowercase dot-separated namespace
segments under their owner package id, so feature selection and export reports
cannot inherit display-style or malformed feature identities.
Static package
`category` values are constrained to the current package families
(`asset_importer`, `authoring`, `diagnostics`, `platform`, `rendering`,
`runtime`, `sdk`), and `maturity` values are constrained to the runtime enum
spellings. `supported_targets` and module `target_modes` must use the known
runtime target spellings `client_runtime`, `server_runtime`, or `editor_host`.
Runtime and editor module names must end with `.runtime` or `.editor`
respectively, keeping module identity readable before projection.
Every declared module target mode must also be covered by package-level
`supported_targets`, so package availability cannot be narrower than its
runtime/editor/native/VM module rows. Editor module rows (`kind = "editor"`)
must target only `editor_host`, so editor-only crates cannot be projected into
client or server runtime package availability. Runtime and editor module
`capabilities` must also match their module kind with `runtime.*` and
`editor.*` namespaces respectively. Every module capability must also stay
under the declaring package or optional-feature namespace, so module rows cannot
claim another plugin's local capability surface.
Package, optional-feature, and module string arrays reject duplicate entries for
targets, capabilities, target modes, and default packaging so repeated metadata
cannot leak into availability or export diagnostics.
Capability-like fields across package, optional-feature, module, dependency,
importer, option, and status rows must use lowercase dot-separated namespace
segments. Package-level and optional-feature capabilities also have one global
static owner, and capability status rows must reference capabilities declared
by the same manifest before readiness diagnostics consume them.
Capability status rows must publish non-empty capability ids, use known status
spellings (`complete`, `partial`, `stub`, `externalized`, `unsupported`), stay
unique per manifest capability, and keep optional target/reference arrays
non-empty and de-duplicated when present. Optional `bevy_references` entries
must be repository-relative paths under `dev/bevy` and resolve to existing
files, so package maturity/status traceability cannot point at stale upstream
source locations.
Package-level dependency rows must publish non-empty `id` and `capability` plus
an explicit `required` marker before plugin availability or editor/export
diagnostics consume them; duplicate package dependency `id` + `capability`
pairs are rejected inside each manifest. Package dependency `id` values and
optional-feature dependency `plugin_id` values must also use trimmed lowercase
token names, so dependency providers cannot drift into display labels or
malformed package identifiers.
Dependency rows that name another static plugin package must reference a
capability declared by that target package or one of its optional features;
dependency rows that name a host-owned provider must use `runtime.module.*` or
`runtime.capability.*` namespaces.
Optional feature rows must also declare matching `owner_plugin_id`, non-empty
owner-namespaced `id`, `display_name`, `capabilities`, known
`default_packaging`, and explicit `enabled_by_default`. Their dependency rows
must publish non-empty
`plugin_id` and `capability`, an explicit `primary` marker, and exactly one
primary dependency pointing back to the owning package capability; duplicate
optional-feature dependency `plugin_id` + `capability` pairs are rejected per
feature. Option `required_capability` gates and importer
`required_capabilities` rows must likewise resolve to a declared static
package/feature capability or an explicit host-owned capability before editor
configuration or importer selection consumes them. Asset importer ids must use
trimmed lowercase dot-separated namespace segments before project import
selection, plugin-window projection, or export diagnostics consume importer
metadata. The same regression family
also checks every declared package and
optional-feature module row for non-empty identity fields (`name`, supported
`kind`, and `crate_name`), non-empty string-array `capabilities` and known
`target_modes`, manifest-local unique module names, module names that use
trimmed lowercase dot-separated namespace segments under the package or
optional-feature namespace, runtime/editor module names that end with the
matching `.runtime` or `.editor` suffix, and `crate_name` values that resolve
to package names in the `zircon_plugins` workspace under the declaring package
root.
Runtime/editor module capabilities must stay in the matching `runtime.*` or
`editor.*` namespace, so runtime/editor/native/VM module projection stays
explicit. Module capability rows must also include the same package or
optional-feature namespace before loader, plugin-window, availability, or
export projections consume them.

Runtime crates depend only on runtime contracts. Editor crates may depend on
`zircon_editor` plus their matching runtime crate when one exists.

The runtime-backed package set is `physics`, `sound`, `texture`, `net`,
`navigation`, `particles`, `animation`, `terrain`, `tilemap_2d`,
`prefab_tools`, `rendering`, `virtual_geometry`, `hybrid_gi`, `solari`,
`gltf_importer`, `obj_importer`, `texture_importer`, `audio_importer`,
`shader_wgsl_importer`, and `ui_document_importer`.
The editor-only package set is `material_editor`, `timeline_sequence`,
`animation_graph`, `runtime_diagnostics`, `ui_asset_authoring`, and
`native_window_hosting`, `editor_build_export_desktop`, and
`plugin_sdk_examples`. Low-overlap diagnostics, platform, and SDK-example
editor-only packages explicitly declare experimental maturity in their static
manifests so availability and export diagnostics do not depend on parser
defaults. `material_editor`, `timeline_sequence`, `animation_graph`, and
`ui_asset_authoring` also declare explicit experimental maturity for the
editor-only authoring package family.

`physics` and `animation` are hard-cut runtime/editor plugin packages in this
workspace. Their runtime crates own the concrete managers, module descriptors,
and scene hooks, while `zircon_runtime` keeps shared framework DTOs, manager
service names/resolvers, scene ECS authority, and the generic scene hook
protocol. Catalog, export, and editor enablement treat both packages as external
plugin packages.

These packages are deliberately kept outside the root workspace so normal
runtime/editor checks only build the minimal core unless an export profile or
plugin CI job selects this workspace.

`texture` is a stable runtime/editor package for texture processing capability.
Its static manifest publishes runtime category, client/editor target set,
primary runtime capability, and complete capability status that match the
built-in catalog and descriptor-derived package manifest used by availability
and export planning.

`sound` is a beta runtime/editor package for runtime audio and editor sound
authoring capability. Its static manifest publishes runtime category,
client/editor target set, and primary `runtime.plugin.sound` capability that
match the built-in catalog and descriptor-derived package manifest before
optional timeline and ray-traced reverb feature rows. Its `sound.dynamic_events`
catalog also publishes concrete Impact, Marker, and Ambient Stinger event rows
for plugin-window, runtime-report, and export metadata consumers.

`net` is a beta runtime/editor package for server/client networking capability.
Its static manifest publishes the same server/client/editor package target set
and primary runtime capability that the built-in catalog and linked runtime
descriptor use for availability, optional feature dependency checks, and export
planning, while its module rows keep runtime and editor targets explicit. Its
`net.runtime_events` catalog now exposes concrete listener, connection, HTTP
route, and WebSocket frame event rows instead of an empty catalog placeholder.

`navigation` is a beta runtime/editor package for gameplay navmesh and path
query capability, separate from UI focus/navigation parity. Its static manifest
publishes the same client/server/editor target set and primary runtime
capability that the built-in catalog uses for package availability and export
planning.

`particles` is an experimental runtime/editor package for optional particle and
VFX capability. Its static manifest publishes the same client/editor target set
and primary runtime capability that the built-in catalog uses before optional
feature dependency checks and explicit feature packaging rows for physics,
animation control, and GPU simulation. Its `particles.dynamic_events` catalog
declares spawn-once, begin-emission, and end-emission rows used by authoring and
runtime report metadata.

`virtual_geometry` and `hybrid_gi` are experimental rendering packages. Their
static manifests and built-in catalog descriptors publish rendering category,
client/editor target set, primary runtime capability, and advanced render
capability rows so export planning and quality-profile filtering see the same
metadata shape before module-local declarations.

The Authoring plugin family follows the same package rules: `terrain`,
`tilemap_2d`, and `prefab_tools` own runtime asset/component descriptors and
publish client/editor package targets plus their primary runtime capability at
the static package level. `material_editor`, `timeline_sequence`,
`animation_graph`, and `ui_asset_authoring` are editor-only authoring packages
over existing runtime asset contracts.

Static package layout metadata is now guarded before export planning and asset
discovery consume it: declared `supported_platforms` must use known
`ExportTargetPlatform` names with no duplicates, and declared `asset_roots` or
`content_roots` must be relative, forward-slash package paths with unique,
non-empty entries.
When a manifest uses reverse-DNS package coordinates, it must declare
`package_prefix`, `package_company`, and `package_name` together using trimmed,
lowercase coordinate segments; the resolved `package_id()` value is also
guarded globally so export reports and plugin registries cannot collide on the
coordinate-derived id.

The importer package split now has root-level runtime packages for
`gltf_importer`, `obj_importer`, `texture_importer`, `audio_importer`,
`shader_wgsl_importer`, and `ui_document_importer`. These packages publish
package manifests, runtime module declarations, capability-gated
`AssetImporterDescriptor` rows, runtime selections, and registration reports.
Static importer rows are guarded for globally unique importer ids, matching
owner package ids, integer priority/version fields, known `ResourceKind`
output names, normalized source-extension or full-suffix matchers, and unique
required capability lists before the asset pipeline consumes them.
The older `asset_importers/{model,texture,audio,shader,data}` family crates
remain in the workspace as declaration aggregators until downstream catalog and
project-selection callers are migrated to the new package ids.

`rendering` is an umbrella runtime/editor package. Its optional feature bundles
live under `rendering/features/<feature_id>/{runtime,editor}` and own
`post_process`, `ssao`, `decals`, `reflection_probes`, `baked_lighting`,
`ray_tracing_policy`, `shader_graph`, and `vfx_graph`. The runtime crate stays
metadata-only; individual feature crates register render descriptors, executor
ids, component descriptors, or local graph compiler DTOs. Its static manifest
publishes rendering category, stable maturity, client/editor package targets,
and the primary `runtime.plugin.rendering` capability before optional feature
rows.

`solari` is an experimental rendering-category runtime package. Its linked
runtime descriptor, static `plugin.toml`, and built-in catalog metadata all
publish client/editor targets, runtime Solari capabilities, experimental
maturity, and the partial status note for the unavailable realtime pass
executor.

`editor_build_export_desktop` is an editor-only package for the desktop export
panel, SourceTemplate/LibraryEmbed/NativeDynamic report templates, and the menu
operations that drive host-owned export planning. `plugin_sdk_examples` is the
SDK fixture package: it contributes a sample editor window plus a sample model
importer, inspector, component drawer, and asset creation template without
requiring runtime linkage.
Static `[[options]]` rows are guarded for globally unique lowercase
dot-separated namespace keys, trimmed display/default strings, known option
value types, default values that parse according to their declared type, and
non-empty optional capability gates before runtime reports or editor
configuration panels consume them.
Static `[[event_catalogs]]` rows are guarded for globally unique lowercase
dot-separated catalog namespaces, positive manifest versions, explicit non-empty
`events` arrays, and lowercase namespace-scoped event ids before event
reflection or runtime reports consume catalog metadata. Optional event
`payload_schema` ids must also use lowercase dot-separated namespace segments so
runtime reports and plugin-window diagnostics do not inherit display labels or
free-form schema names, and they must end with an explicit positive version
segment such as `v1` before schema metadata is accepted.

`native_dynamic_fixture` is the SDK fixture for the NativeDynamic ABI loader.
Its static `plugin.toml` and embedded native-library manifest both publish the
same experimental SDK package category, client/server/editor targets,
runtime/editor capabilities, and `native_dynamic` default packaging because the
native loader reads the embedded TOML from the compiled library.

Editor-only packages that have no runtime manifest still need package-level
metadata, not only module-local rows. Static `plugin.toml`, linked
`package_manifest()` output, and `EditorPluginDescriptor` catalog projection
should agree on package `category`, `supported_targets = ["editor_host"]`, and
the editor capability set so plugin-window and export consumers see the same
package shape.
