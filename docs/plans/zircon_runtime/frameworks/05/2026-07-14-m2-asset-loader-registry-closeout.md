# Frameworks05 M2 AssetLoaderRegistry closeout

Plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
Milestone: M2
Status: completed
Files: ["docs/plans/zircon_runtime/frameworks/05/2026-07-14-m2-asset-loader-registry-closeout.md", "zircon_plugins/ui_document_importer/runtime/Cargo.toml"]

## Scope Delivered

- The existing `AssetImporterHandler` and `AssetImporterRegistry` remain the only generic importer extension contract; M2 does not introduce a second registry or compatibility facade.
- The `.zui` descriptor and importer registration are owned by `zircon_plugins/ui_document_importer/runtime`; the retired asset-domain builtin importer owner is absent.
- Asset UI DTO handling remains local to `zircon_runtime/src/asset/assets/ui/{document_loader,resource_references}.rs` and does not import the UI implementation domain.
- The former crate-root declaration-order comment is absent, so module declaration order no longer carries asset/UI behavior.

## Fresh Testing Evidence

- Windows static boundary gate: `python -m unittest tools.tests.test_frameworks_05_asset_ui_boundary -v` passed 3/3 on 2026-07-14. The only raw `crate::ui` search hit is test support at `asset/tests/support.rs`; the production-only guard correctly excludes that test owner.
- Current production dependency audit reports 2,323 references / 76 domain edges and no `asset -> ui` matrix entry, so the production edge remains zero.
- `cargo metadata --format-version 1 --no-deps --locked --manifest-path zircon_plugins/ui_document_importer/runtime/Cargo.toml` confirms the plugin resolves `zircon_runtime` with `default_features=false` and `features=["ui"]`. The plugin unconditionally imports `zircon_runtime::ui::v2::UiZuiAssetLoader`, so this is the explicit required feature contract rather than a compatibility feature.
- Managed Windows UI document importer package tests compiled the Runtime UI surface and passed 7/7; doc-tests passed 0/0. The terminal log is `E:/ZirconBuilds/frameworks05-m2-ui-document-importer-test-20260714.log`.
- Managed Windows compile gate passed with `CARGO_INCREMENTAL=0`: coordinator job `95ef83ecd491435888e803b867f52493`, `cargo build -p zircon_runtime --locked`, exit 0 in 10m11s. This compiled the lib and package binaries, exceeding the plan's lib-only check surface.
- The exact managed lib command `cargo test --package zircon_runtime --lib --locked` completed under coordinator job `5de2ad02b6ed49208957b80d23e7b4b1`: 7,792 passed, 229 failed, 35 ignored. The failure set is cross-plan and environmental rather than M2-local: Windows link/reparse tests lack symlink privilege, parallel WGPU tests report device OOM, and active Shader/Render/UI/Text/structure changes invalidate multiple expectations. The complete log is `E:/ZirconBuilds/frameworks05-m2-runtime-lib-test-20260714.log`; this record does not claim a green Runtime package.
- Fresh serial rerun job `1983283912904d59a4855dfb2abba56a` became coordinator `orphaned` after 3,611 seconds with no exit code or final test summary. Before termination it recorded 6,439 passed, 130 failed, and 36 ignored out of 8,063 tests; no completed failure names the `.zui` importer or the M2 asset/UI boundary. This partial run is diagnostic only and does not replace the complete terminal run above.
- The M2 candidate changes only the external UI document importer plugin manifest, which is not an input to the `zircon_runtime --lib` test target. Runtime package failures therefore remain broader-worktree diagnostics, while the directly affected plugin compile/registration/parse surface is covered by the 7/7 package result and the 3/3 boundary suite.
- The plan-output audit reports five foreign-owner violations (Editor UI 01/10/11, Editor UI index, and Plugins05); no finding targets Frameworks05 or this M2 output.

## Review

- The workspace dependency disables Runtime default features, while the importer source unconditionally consumes `UiZuiAssetLoader`; enabling exactly `ui` is necessary and sufficient for this plugin package.
- Runtime does not depend on the UI document importer plugin, so the manifest change does not introduce a crate cycle or move `.zui` registration back into the asset domain.
- Final coordinator review is submitted from a distinct reviewer Session after this two-file manifest is frozen. Acceptance requires 0 Critical and 0 Important findings.

## Status And Completed Items

| Milestone | Item | Status | Evidence |
|---|---|---|---|
| M2 | AssetLoaderRegistry single extension boundary | completed | `.zui` registration is plugin-owned; no parallel registry or compatibility owner exists. |
| M2 | Asset-to-UI production edge | completed | Static boundary suite passed 3/3 and the production matrix reports zero edges. |
| M2 | Declaration-order semantic dependency | completed | The old crate-root order comment is absent and guarded. |
| M2 | M2-T testing stage | completed | Runtime build passed; importer package tests passed 7/7; boundary suite passed 3/3; complete Runtime lib-test failures and the later one-hour orphaned rerun are explicitly retained as external degraded-baseline diagnostics. |
