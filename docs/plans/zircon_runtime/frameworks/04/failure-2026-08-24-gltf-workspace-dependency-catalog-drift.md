---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-24
summary_slug: gltf-workspace-dependency-catalog-drift
origin_plan: docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
fixing_plan: docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/04
fixing_child_dir: docs/plans/zircon_runtime/frameworks/04
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/Cargo.toml
  - zircon_plugins/gltf_importer/runtime/Cargo.toml
plan_sources:
  - docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
  - docs/plans/engine-code-structure-convention.md
---

# Frameworks04 glTF workspace dependency catalog drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md`
- 来源执行切片：Plugins04 glTF workspace dependency catalog validation
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md`
- 交接原因：Frameworks04 owns the plugin workspace dependency catalog and the
  managed compile gate that exposed this local catalog drift.

## 失败现象与复现证据

`source_implemented / validation_blocked`

Managed Plugins04 Cargo job `bdeb93ea01094ea1aeb6412cdc2aa2f9` stopped before compiling the owned animation slice because `zircon_plugins/gltf_importer/runtime/Cargo.toml` had already hard-cut its `gltf` dependency to `workspace = true`, while `zircon_plugins/Cargo.toml` did not declare `gltf` in `[workspace.dependencies]`.

The consumer blob was pre-existing and unowned when this failure session started. Its admitted hash was `6d92eeed07c4aa0aae61d5f7c4a9578805c9e3b4`; this session does not rewrite that blob.

## 最低共享层根因

The workspace consumer and the workspace dependency catalog had diverged. The
consumer correctly required a canonical workspace dependency, but the root
plugin workspace did not provide the corresponding `gltf` entry, so Cargo
manifest resolution failed before the owned slice could compile.

## 架构修复验收

- Keep the consumer on `workspace = true`; do not restore a crate-local compatibility declaration.
- Add the canonical workspace dependency at exact version `1.4.1`.
- Preserve the former `KHR_texture_transform` and `extensions` feature contract in the workspace declaration.
- Leave both lockfiles unchanged because they already resolve `gltf`, `gltf-derive`, and `gltf-json` at `1.4.1`.

- Static TOML contract: PASS. Python `tomllib` confirmed workspace version `1.4.1`, exact preserved features, and the unchanged optional workspace consumer.
- Scoped `git diff --check`: PASS; only line-ending conversion warnings were emitted.
- Managed Windows Cargo job `0f5153124e5e4be583a2976337a47923` reached `cargo build` and therefore proved the former missing-workspace-dependency parse failure is gone. It then stopped with exit 101 because the plugin workspace lock requires an unrelated update.
- The unrelated lock drift is owned by Frameworks05: `zircon_plugins/first_party_runtime_catalog/Cargo.toml` adds `zircon_plugin_ui_document_importer_runtime`, while the `zircon_first_party_runtime_catalog` entry in `zircon_plugins/Cargo.lock` does not list it. Active session `frameworks05-zui-importer-provider-linkage-regression-r4-20260823` owns the catalog manifest and root `Cargo.lock`, but its immutable scope omits `zircon_plugins/Cargo.lock`; that owner needs a scope rotation before the locked gate can pass.
- Coordinator validation-copy job `a1c6573223754b80ac90fcc471984ac6` failed before Cargo at `owned_overlay` with `validation_copy_owned_source_reappeared`; no isolated result is claimed.
- Pending: rerun the focused Plugins04 compile that first exposed this failure.
- Pending: independent review, coordinator integration commit, and WeCom milestone notification.

## 禁止临时方案

- Do not restore a crate-local `gltf` dependency or weaken the workspace
  dependency hard cut merely to bypass the catalog contract.
- Do not modify either lockfile under this failure when the remaining lock drift
  belongs to another active owner.
- Do not claim the blocked compile as a Frameworks04 milestone or a successful
  product gate.

## 修复结果与回传

Open state: `source_implemented / validation_blocked`. The exact workspace
dependency entry and preserved features have static proof, and managed job
`0f5153124e5e4be583a2976337a47923` proved Cargo passed the original missing
dependency parse boundary. The unrelated Frameworks05 lock drift and the final
focused Plugins04 compile remain pending. No Frameworks04 milestone is promoted
by this source repair alone.

### 2026-09-24 current plugin-catalog lock blocker

The original glTF workspace declaration still resolves from
`zircon_plugins/Cargo.toml` as version `1.4.1` with
`KHR_texture_transform` and `extensions`; its optional importer consumer
still uses `workspace = true`. A read-only Python `tomllib` comparison of the
current `zircon_plugins/first_party_runtime_catalog/Cargo.toml` with the
`zircon_first_party_runtime_catalog` entry in `zircon_plugins/Cargo.lock`
shows that the previously missing UI document importer is now listed, but
the current catalog manifest has **21 direct dependencies against 19 locked
dependencies**. The lock entry is still missing exactly
`zircon_plugin_net_http_runtime` and
`zircon_plugin_net_websocket_runtime`. Both packages have their own lock
entries, and the catalog's current `base-runtime-plugins` feature and
`first_party_net_feature_registrations` production path actually use them.

These are live, dirty, unattributed catalog/lockfile changes outside the
Frameworks04 glTF-owner scope: manifest SHA-256
`54358512bb750e41931285831b686e845ddde1f4756d971f2d91b33540395c8f`,
lock SHA-256 `35033c3ed8892a880f38be02ef0c58faece418d1c9d1aa7b228756e04dbb5c1f`.
The earlier Frameworks05 handoff for the catalog provider is
[recorded here](../05/failure-2026-08-23-zui-importer-provider-linkage-regression.md),
but neither this static check nor the historical glTF job establishes a new
`--locked` compile pass. The catalog/lock owner must establish source
attribution and reconcile the remaining two dependencies in its own scope;
Frameworks04 then reruns the original focused Plugins04 compile on a managed,
exact-source Windows snapshot. No lockfile edit, fixed return, closeout, or
product test pass is claimed here.
