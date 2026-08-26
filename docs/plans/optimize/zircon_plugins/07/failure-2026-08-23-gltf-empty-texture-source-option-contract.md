---
handoff_kind: failure
status: open
created_at: 2026-08-23
summary_slug: gltf-empty-texture-source-option-contract
origin_plan: docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md
fixing_plan: docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
origin_child_dir: docs/plans/optimize/zircon_plugins/09
fixing_child_dir: docs/plans/optimize/zircon_plugins/07
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
tests:
  - cargo +1.94.1 check -p zircon_plugin_gltf_importer_runtime --locked --jobs 1
  - cargo +1.94.1 test -p zircon_plugin_gltf_importer_runtime --locked --jobs 1 -- --test-threads=1
  - cargo +1.94.1 test -p zircon_app --lib --features first-party-runtime-plugins --locked runtime_profile_bootstrap --jobs 1 -- --test-threads=1
---

# Plugins07: glTF empty-texture source option contract

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md`
- 来源执行切片：Particles neutral identity hard cut 的 Frameworks05 exact App upward gate
- 修复责任计划：`docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md`
- 交接原因：App gate 已越过 Runtime、ZUI importer provider、Particles 和 Sound，随后只被 glTF importer
  的 current-source compile error 阻断。目标文件不在 Plugins09 immutable scope，并含另一会话尚未提交的
  material-dependency blob，不能由本会话覆盖或拆分。

## 失败现象与复现证据

Managed job `a523910f7a204dc28d3e15461103e9f4` / run
`b575c7a8217743adbe0e5b124c1cb9b7` 在 D 盘 retained pool 执行：

`cargo +1.94.1 test -p zircon_app --lib --features first-party-runtime-plugins --locked runtime_profile_bootstrap --jobs 1 -- --test-threads=1`

Cargo 在 2026-08-23 19:17:24 +08:00 以 exit `101` 终止。精确错误为
`zircon_plugins/gltf_importer/runtime/src/subassets.rs:37:44` E0599：
`texture.source()` 返回 `Option<gltf::Image<'_>>`，当前代码却直接调用 `.index()`。

Coordinator ownership matrix request `0cea2d3236f74291835d5d083698f063` 记录 current blob hash
`eda8bce6c5b06fb42f55c9676a2c90f1df2b646460ff1631fdc8442865e839e9`；旧 attribution 指向
`optimize-plugins07-texture-sampling-r1-f1614c5e-20260823`，但 baseline stale 且无 live lease。修复会话必须
以该整 blob hash 为输入重新取得合法 owner，保留其中现有 material dependency 改动。

## 最低共享层根因

- workspace 使用 `gltf 1.4.1`。其 `Texture::source` 在 `allow_empty_texture` feature 下返回
  `Option<Image>`；未启用该 feature 时才返回裸 `Image`。first-party App feature 图合并后前者生效，因此
  standalone assumptions 不能代表产品闭包。
- `None` 是可表示的 empty texture source，不应通过 `expect`、`unwrap` 或默认到 image 0 继续导入。
  importer 应在发布任何 texture subasset 前返回包含 texture index 的 `AssetImportError::Parse`，保持
  fail-closed、零伪造引用和确定性诊断。
- 当前函数已返回 `Result<AssetImportOutcome, AssetImportError>`，最低 owner 可直接表达 typed failure；
  不需要修改 App feature、关闭 first-party glTF provider、降级 glTF crate 或增加兼容 facade。

## 架构修复验收

- `texture.source()` 的 `None` 在 `add_gltf_texture_subassets` 最低 owner 转为确定性 typed parse error，
  诊断包含 texture index；禁止 panic、silent skip、image 0 fallback 或部分发布。
- 增加 focused negative regression，真实构造或解析 empty-source texture，并断言 exact failure category/message
  及零 texture subasset publication。保留 current blob 的 material dependency 行为与对应 tests。
- managed glTF package check/tests GREEN；随后重跑 exact App command，必须实际执行
  `runtime_profile_bootstrap` tests，而不只是越过编译。
- scoped rustfmt、diff-check、hard-cut/static guards 与独立 source review通过后，才可回传 fixed。

## 禁止临时方案

- 不得关闭 `allow_empty_texture`、`extensions` 或 first-party runtime plugins feature 来恢复旧签名。
- 不得用 `expect`/`unwrap`、synthetic image、test-only cfg 或忽略 texture 来掩盖缺失 source。
- 不得覆盖、拆分或回退 current `subassets.rs` 中 foreign material-dependency blob；必须按整 blob ownership
  transfer/新 owner 会话继续。

## 修复结果与回传

- 当前状态保持 `open`：Plugins07 尚需完成 typed empty-source 修复、focused negative regression 与受管验证。
- 修复方只有在上述架构验收全部通过后，才可将本记录转为 `fixed` 并向来源计划回传终态证据。
