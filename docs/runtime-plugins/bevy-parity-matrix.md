---
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor_builder.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/capability_status.rs
  - zircon_runtime/src/plugin/plugin_maturity.rs
  - zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/net/plugin.toml
implementation_files:
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor_builder.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/capability_status.rs
  - zircon_runtime/src/plugin/plugin_maturity.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/net/plugin.toml
plan_sources:
  - user: 2026-05-08 实现 ZirconEngine Bevy 级插件完成度里程碑计划
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
tests:
  - zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs
  - cargo test -p zircon_runtime --locked plugin_extensions::profile_maturity -- --nocapture
  - cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1
  - cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1
doc_type: module-detail
---

# Bevy Parity Matrix

This matrix records the M0/M1 plugin metadata layer. It does not claim feature completion for animation, sound, navigation, networking, or particles; it makes their current maturity and capability status explicit so profiles and export can gate them without parsing generic externalized warnings.

## Metadata Model

`PluginMaturity` is now carried by both `RuntimePluginDescriptor` and `PluginPackageManifest`:

- `Core`: built into the runtime profile spine.
- `Stable`: suitable for stable/default profile membership.
- `Beta`: first-party runtime exists but product coverage is still partial.
- `Experimental`: advanced optional capability or high-cost feature.
- `Externalized`: descriptor exists but no linked/native registration was supplied.
- `Stub`: descriptor is a placeholder only.
- `Deprecated`: not valid for new required profile use.

`CapabilityStatusManifest` records per-capability state with `Complete`, `Partial`, `Stub`, `Externalized`, or `Unsupported`, optional target modes, Bevy source references, and notes. Package TOML files can persist the same metadata under `[[capability_statuses]]`.

## Current First-Party Baseline

| Plugin | Maturity | Capability Status | Profile Role |
|---|---:|---:|---|
| `rendering` | Stable | `runtime.plugin.rendering = Complete` | Required by `client_2d`, `client_3d`, `editor`, and `dev` profiles. |
| `texture` | Stable | `runtime.plugin.texture = Complete` | Default optional baseline for client/editor profiles. |
| `sound` | Beta | `runtime.plugin.sound = Partial` | Required by client/editor profiles, but still needs M4 product completion. |
| `animation` | Beta | `runtime.plugin.animation = Partial`; timeline event capability partial | Optional profile capability until M3 completion. |
| `navigation` | Beta | `runtime.plugin.navigation = Partial` | Advanced optional gameplay navmesh; UI navigation parity remains separate. |
| `net` | Beta | `runtime.plugin.net = Partial` | Dev/server optional baseline; remote protocol completion is M7. |
| `particles` | Experimental | `runtime.plugin.particles = Partial` | Advanced VFX optional; not a Bevy default parity blocker. |
| `physics`, `virtual_geometry`, `hybrid_gi` | Experimental | Partial | Optional advanced systems outside Bevy default plugin parity. |

Asset importer plugins (`gltf_importer`, `obj_importer`, `texture_importer`, `audio_importer`, `shader_wgsl_importer`, `ui_document_importer`) are marked Stable at package level with Partial importer capability status because the importer surfaces exist but broader asset-stack completion is owned by a separate asset milestone.

## App Provider Closure

The M2 linked-provider closure lives in `zircon_app`, not in `zircon_runtime`. Runtime profiles and catalog descriptors remain implementation-agnostic, while `zircon_app` can opt into compiled first-party providers through feature-gated dependencies on `zircon_plugins/*/runtime` crates. The `first-party-runtime-plugins` app feature links `sound`, `rendering`, `texture`, `animation`, `net`, and `particles`; `first-party-navigation-runtime-plugin` links `navigation` separately because its runtime crate builds the Recast/Detour native bridge.

Profile bootstrap tests cover the practical Bevy-grade path: `EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d)` projects the profile manifest, the app provider supplies linked registration reports for required sound/rendering and optional texture, and `BuiltinEngineEntry` appends the resulting plugin modules without making `zircon_runtime` depend on any `zircon_plugin_*` crate.

The same app-provider validation now covers app-owned render/platform bootstrap persistence. Headless profile selection stores both the headless platform feature set and the `Headless` render bundle, while the provider-enabled `client_2d` path keeps linked first-party registration closure in `zircon_app`.

## Bevy References

The metadata intentionally points to local Bevy source files rather than copying API shapes:

- `dev/bevy/crates/bevy_audio/src/lib.rs` for audio plugin expectations.
- `dev/bevy/crates/bevy_animation/src/lib.rs` for animation plugin and timeline event precedent.
- `dev/bevy/crates/bevy_remote/src/lib.rs` for remote/dev network-facing tooling.

Navigation and particles diverge deliberately: Bevy first-party navigation in the referenced plan is UI focus/directional navigation, while Zircon's current `navigation` plugin is gameplay navmesh/pathfinding. Bevy core has no first-party particles crate, so Zircon particles stay experimental optional.

## Validation

The profile/maturity unit tests cover manifest serde roundtrip, descriptor-to-manifest projection, built-in catalog classification, profile ids, deterministic profile manifests, required-plugin availability gates, optional-plugin warning buckets, and provider-aware linked/native registration buckets.

Review-fix coverage now also pins exact profile-manifest loading for `minimal`, provider reports not bypassing `Stub` or below-minimum maturity gates, and built-in importer/physics capability statuses matching the persisted plugin TOML capability metadata.

Latest attempted validation in this session:

- `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture` passed: 8 profile/maturity tests, 0 failures.
- `cargo test -p zircon_runtime --lib plugin_extensions::manifest_contributions::builtin_runtime_catalog_entries_have_matching_plugin_manifests_and_workspace_members --locked -- --nocapture` passed: 1 manifest/catalog matching test, 0 failures.

Latest M2 scoped validation:

- `cargo check -p zircon_runtime --lib --locked` passed with warning-only output.
- `rustfmt --edition 2021 --check <scoped profile/maturity files>` passed after formatting scoped files.
- `git diff --check -- <scoped profile/maturity files and docs>` passed with line-ending normalization warnings only.
- `cargo test -p zircon_runtime --lib plugin_extensions::manifest_contributions::builtin_runtime_catalog_entries_have_matching_plugin_manifests_and_workspace_members --locked -- --nocapture` passed: 1 manifest/catalog matching test, 0 failures.
- `CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" entry_config_can_select_headless_render_profile_bundle -- --nocapture --test-threads=1` passed: 1 test, 0 failures.
- `CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1` passed: 15 tests, 0 failures.
- `CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 profile_bootstrap -- --nocapture --test-threads=1` passed: 13 tests, 0 failures.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/opencode/zircon-profile-provider-target cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1` passed in WSL/Linux: 1 test, 0 failures. The earlier Windows attempt stopped in `wgpu-hal` before provider code because `windows 0.61.3` and `windows 0.62.2` D3D12 types were both present in the root dependency graph.
