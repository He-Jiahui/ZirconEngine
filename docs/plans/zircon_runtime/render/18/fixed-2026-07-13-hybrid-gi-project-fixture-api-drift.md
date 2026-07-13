---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: hybrid-gi-project-fixture-api-drift
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_editor/editor/10
related_code:
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_scene_prepare_material_fixtures.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_scene_prepare_material_fixtures/project_documents.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources/runtime_prepare_collector.rs
  - zircon_plugins/hybrid_gi/runtime/Cargo.toml
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/model/model_asset.rs
  - zircon_runtime/src/asset/assets/scene/asset.rs
tests:
  - python -m unittest tools.tests.test_hybrid_gi_m4_contract tools.tests.test_hybrid_gi_editor_profile
  - validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime -SkipTest
  - validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime
resolved_at: 2026-07-13
---


# Editor 10：HybridGI project fixture API drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行切片：Hybrid GI M4 crate behavior testing stage
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：首次复现时，HybridGI fixture 同时落后于 Editor10 的显式多资产根和项目文档硬切；Render18 不得恢复固定资产根或测试专用兼容接口。2026-07-13 复审确认文档 writer 已公开，剩余迁移只位于 Render18 fixture 消费方，但继续沿用本 lifecycle key 直到 upward gate 实际通过。

## 失败现象与复现证据

`zircon_plugin_hybrid_gi_runtime` 测试目标编译曾在
`hybrid_gi_scene_prepare_material_fixtures.rs` 稳定报告以下 API 漂移：

- `ProjectPaths::assets_root()` 已硬删除，资产根必须由 `asset_root(&RelPath)` 解析。
- `ProjectPaths::ensure_layout()` 已改为接收显式 `&[RelPath]`。
- 历史复现还报告 material/model/scene TOML writer 可见性；三个无 resolver 的
  `to_toml_string()` 仅对 Runtime 自己的 `cfg(test)` 开放，跨 crate fixture 必须使用公开的
  `to_project_toml_string(...)` 并提供项目引用持久化规则。

Render18 已先写静态 RED，证明 fixture 尚未从 `ProjectManifest` 读取 `asset_roots`/`primary_asset_root`，且保留 39 个固定根调用。
当前实现已移除全部旧调用，并在共享桥接 owner 稳定后通过完整 HybridGI official build/test gate。

## 最低共享层根因

Editor10 的多根硬切本身是正确的：`ProjectPaths` 不再拥有隐式固定资产根，project manifest 才是根列表真源。
最低错误是 HybridGI test fixture 没有跟随该合同：它既把所有临时项目写死到退役资产根 API，又跨 crate
调用只对 Runtime `cfg(test)` 开放的无 resolver writer。修复必须让 manifest 同时拥有资产根，并让 fixture
通过公开 project writer 把 `builtin://` 保持为 builtin reference、把 `res://` 投影为 root-relative project reference。

## 架构修复验收

- 四组 fixture 先构造 `ProjectManifest`，通过 `ensure_layout(&manifest.asset_roots)` 创建布局，使用
  `manifest.primary_asset_root()` 取得根，并用 `asset_root(&RelPath)` 解析所有模型、材质、纹理和场景路径。
- material/model/scene 文档通过 `to_project_toml_string` 写入；项目引用的 path hint 来源于同一 manifest root，
  不使用 Runtime 私有测试 writer 或无 GUID 的手写 TOML。
- 全文件不再包含 `.assets_root()` 或无参数 `.ensure_layout()`，且不增加 alias、shim 或本地重复路径真源。
- HybridGI static contracts、standalone production build 与 crate test compile 依次通过。
- 执行 scene/material/provider 行为测试后，由 coordinator 将同一 artifact 返回 Render18 的 `18/` 子目录。

## 禁止临时方案

- 禁止恢复固定 assets 根、零参数 `ensure_layout`、兼容 shim、test-only bypass 或重复 ProjectPaths。
- 禁止把 fixture 改为手写绕过 ProjectManager/AssetManager 的伪加载路径。
- 禁止在任一下层共享编译失败仍存在时把静态合同外推为 Rust behavior GREEN。

## 修复结果与回传

- 根因：HybridGI fixture still depended on retired implicit asset roots and runtime-crate-only cfg(test) document writers.
- 架构修复：Migrated all fixtures to manifest-owned asset roots and public project-aware TOML writers with root-relative persisted references; split document persistence into project_documents.rs without shims.
- 验证：Static contracts 13/13; standalone official production build exit 0; complete crate test phase passed 124 tests with 0 failed and 0 ignored; rustfmt and scoped diff checks passed.
- 回传：Editor10 project fixture API drift is fixed and returned to Render18; Runtime02 Editor startup and active Text05 SM4 compilation remain independent upward-gate work.
