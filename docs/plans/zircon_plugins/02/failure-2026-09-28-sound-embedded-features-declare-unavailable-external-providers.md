---
handoff_kind: failure
status: open
created_at: 2026-09-28
summary_slug: sound-embedded-features-declare-unavailable-external-providers
origin_plan: docs/plans/astra/features/plugins/01-selection-and-product-eligibility.md
fixing_plan: docs/plans/zircon_plugins/02-sound.md
origin_child_dir: docs/plans/astra/features/plugins/01
fixing_child_dir: docs/plans/zircon_plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/src/runtime_plugin/feature_manifest.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/plugin.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features/rows.rs
  - zircon_runtime/src/plugin/export_build_plan/linked_feature_source.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_feature_provider.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter linked_feature_export_without_source_root_or_independent_manifest_fails_closed
---

# Plugins 02: Sound embedded features declare unavailable external providers

## 来源执行者

- 来源计划：`docs/plans/astra/features/plugins/01-selection-and-product-eligibility.md`
- 来源执行切片：M3 linked feature source role admission.
- 修复责任计划：`docs/plans/zircon_plugins/02-sound.md`
- 交接原因：Sound owns the two feature declarations and their physical runtime, editor, and native distribution crates. The shared source admission contract correctly requires evidence for a declared external package.
- This child record carries the open link because the origin plan file is currently an untracked foreign-owned work product. Do not edit or relabel it during this handoff.

## 失败现象与复现证据

`zircon_plugins/sound/plugin.toml` declares `sound.timeline_animation_track` with provider `sound_timeline_animation_track` and `sound.ray_traced_convolution_reverb` with provider `sound_ray_traced_convolution_reverb`. Both IDs differ from owner `sound`. `ProjectPluginFeatureSelection::runtime_crate_path` therefore selects `<provider>/runtime`, and `linked_feature_source::admit_source` requires `<plugin_root>/<provider>/plugin.toml` plus the matching runtime crate.

Read-only reproduction on 2026-09-28 parsed the real Sound manifest and checked the repository paths. For both features, the declared external `plugin.toml` was absent and `sound/features/<feature>/runtime/Cargo.toml` existed. The existing focused test named in frontmatter checks that a nested owner crate cannot satisfy a missing independent provider manifest in a synthetic fixture. It has **not** been executed for this handoff; no managed Cargo or product acceptance is claimed. The current source-root test uses a synthetic external provider, so it does not prove either real Sound feature exports.

The timeline manifest SHA-256 is `07ad5492d99c6dc7318564b084c61de1d5bf59da961008c9ad0b1a5954fe4ab3`. The linked source admission SHA-256 is `1a0c628c9cd85d6a806c87ea323480a01a33b86e369e98052226d513f527e23b` at intake.

## 最低共享层根因

The Sound package declares two independent providers that have no corresponding source packages. Its runtime feature manifests, built-in catalog, nested runtime registrations, native distribution identities, Cargo workspace members, and tests also project those identities. Correct the Sound-owned package layout and all direct projections as one consistent migration. A real independent provider package for each retained external ID must have an admissible manifest and actual runtime crate at the declared path. If an owner-embedded identity is chosen instead, update every producer and consumer, including NativeDynamic behavior, so no path or identity still claims an independent provider.

## 架构修复验收

1. Add and actually execute real-repository source-root tests for both Sound features. Each must prove that source planning admits the declared provider/owner, resolves the generated Cargo dependency to the actual runtime crate, and retains source identity through materialization. Also execute the existing lower-layer negative admission regression.
2. Cover Sound manifest, runtime feature catalog, native distribution, Cargo workspace and direct consumer consistency. Preserve the required Animation timeline and Physics raycast dependencies.
3. Run coordinator-managed Windows `--locked` lower regression, original reproduction, and upward Astra M3 source-template/library-embed export acceptance on the exact attributed snapshot. NativeDynamic acceptance must cover both feature identities if the migration affects them. Compilation products must remain under approved D/E/F drive-root `cargo-targets`.
4. Return this lifecycle only after evidence matches the current source/configuration, independent review has zero Critical, Important, and Moderate findings, and coordinator closeout records its commit SHA and WeCom result.

## 禁止临时方案

- Keep `linked_feature_source` fail-closed: it must not infer an external package's role from a nested crate, a provider ID, a directory name, or a synthetic fixture.
- Do not add a fallback path, alias, symlink, duplicate crate, compatibility shim, or test-only package to make admission appear successful.
- Do not weaken the existing negative test or count a static path check as dynamic acceptance.

## 修复结果与回传

Open state: `待修复`; source ownership transfer, repair, real-repository tests, managed validation, review, return, and closeout remain pending. Source-comment audit owns dirty producer and catalog paths; preserve its comment bytes and attribution until a formal exact-path transfer. This record does not claim a source fix or dynamic pass.
