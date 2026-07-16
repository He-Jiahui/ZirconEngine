---
related_code:
  - zircon_runtime/tests/runtime_environment_reflection_probe_product_contract.rs
  - zircon_runtime/tests/support/project_asset_runtime.rs
implementation_files:
  - zircon_runtime/tests/runtime_environment_reflection_probe_product_contract.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - managed Windows cargo test -p zircon_runtime --locked (job fd638c0b09b24827b92758c886f4f09a; red E0308 at stale reflection-probe integration consumer)
  - managed Windows cargo test -p zircon_runtime --test runtime_environment_reflection_probe_product_contract --locked --offline (job 8f0af97d9f1d4a5a94b72c21a1ceab03; 1 passed / 0 failed)
  - rustfmt --edition 2021 --check zircon_runtime/tests/runtime_environment_reflection_probe_product_contract.rs
  - git diff --check -- zircon_runtime/tests/runtime_environment_reflection_probe_product_contract.rs
doc_type: milestone-detail
---

# Frameworks05 Reflection Probe Product Manager Access Consumer

Plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
Parent milestone: M4
Status: focused_cargo_passed
Date: 2026-07-15
Files: ["docs/plans/zircon_runtime/frameworks/05/2026-07-15-reflection-probe-product-manager-access-consumer.md", "zircon_runtime/tests/runtime_environment_reflection_probe_product_contract.rs"]

## 状态与完成项目

| 切片 | 状态 | 完成证据 |
|---|---|---|
| reflection-probe product integration manager-access hard cut | `focused_cargo_passed` | 当前测试已复用真实 `ProjectAssetTestRuntime`，通过 `ProjectAssetManagerAccess` 构造 render framework，并保持 runtime fixture 覆盖完整测试生命周期。受管 job `8f0af97d9f1d4a5a94b72c21a1ceab03` 实际运行产品契约并得到 `1 passed / 0 failed`。 |

## 根因与修复

`WgpuRenderFramework::new_with_plugin_render_features` 已硬切为
`ProjectAssetManagerAccess`，但 reflection-probe product integration test 仍直接传入
`Arc<ProjectAssetManager>`。生产 contract 正确，最低错误位于测试消费者。

测试继续使用原 concrete manager 注册 material 与 PMREM 资源，同时用同一 manager 创建
`ProjectAssetTestRuntime`。构造 render framework 时只传入 `asset_runtime.access()`；fixture
局部变量保持到测试函数结束，因此 manager service 不会在 frame capture 前失活。

没有增加 constructor overload、Arc adapter、旧 resolver、compatibility shim、双 owner 或
静默 fallback。

## 验证

- job `fd638c0b09b24827b92758c886f4f09a` 提供红态：
  `runtime_environment_reflection_probe_product_contract.rs:44` 报 E0308，期望
  `ProjectAssetManagerAccess`、实际收到 `Arc<ProjectAssetManager>`。
- job `8f0af97d9f1d4a5a94b72c21a1ceab03` fresh 编译并执行 reflection-probe 产品契约，
  `reflection_probe_feature_off_matches_skybox_and_enabled_probes_change_pixels` 为
  `1 passed / 0 failed`，耗时 33.34 秒。
- scoped rustfmt、diff check 与 source invariant scan 通过。
- 同一 job 的 UI text 集成目标仍被活动 Editor Layout 15 native-font 回归阻断；因此本记录只把
  reflection-probe consumer 标记为 focused pass，不把 Frameworks05 M4 或 Runtime15 标记为完成。

## 剩余范围

本记录只覆盖 reflection-probe product integration consumer。Frameworks05 M4、Runtime15
render owner Failure 回传和全 workspace 验证继续按原计划拓扑执行。
