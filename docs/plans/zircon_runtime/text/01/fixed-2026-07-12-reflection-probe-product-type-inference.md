---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: reflection-probe-product-type-inference
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/render/11-environment-lighting.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/render/11
related_code:
  - zircon_runtime/src/graphics/tests/project_render/project_scenes/reflection_probe_product.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_runtime --lib screen_space_ui_font_initialization_discovers_system_faces_from_empty_snapshot --no-default-features --features target-client --locked --jobs 1
resolved_at: 2026-07-12
---


# Render 11：Reflection-probe 产品测试类型推断失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Editor M1 HUD glyph 的 Text 01 下层字体发现复验
- 修复责任计划：`docs/plans/zircon_runtime/render/11-environment-lighting.md`
- 交接原因：Runtime lib-test 在执行 Text exact 前编译全部测试模块，当前最低编译错误位于 Render 11 的 reflection-probe 产品测试。

## 失败现象与复现证据

`zircon_runtime/src/graphics/tests/project_render/project_scenes/reflection_probe_product.rs:175` 的 `.all(|pass| !pass.contains("reflection-probe"))` 无法推断闭包参数类型，返回 E0282；同一 Cargo fingerprint 汇总为 3 errors / 337 warnings，因此没有生成包含新 Text 回归的 runtime test binary。

权威诊断：`F:\cargo-targets\zircon-text-product\debug\.fingerprint\zircon_runtime-a56714e87de94b85\output-test-lib-zircon_runtime`。

## 最低共享层根因

Reflection-probe graph-pass 断言被放入尚未取得 `RenderStats` 的 snapshot 构造阶段，导致闭包参数没有可推断的 pass 名称类型；最低失配位于 Render 11 产品测试内部的数据源边界。

## 架构修复验收

- 在 reflection-probe 产品测试的数据源处恢复明确、共享的 pass 名称类型，不在 Text 测试或 Cargo 调用中绕过该模块。
- 先复验对应 reflection-probe 产品测试编译/断言，再让 Text owner 重跑其字体发现 exact。
- 禁止删除产品断言、改成无类型空集合、添加 cfg 跳过或兼容旧 probe pass 名称。

## 禁止临时方案

- 禁止添加 pass-name alias、空集合 fallback、`cfg` 跳过或只为 Text exact 绕过 reflection-probe 产品模块。
- 禁止删除或弱化“graph 不含未请求 probe pass”的断言。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| EL-M2 | Reflection-probe product pass-name type inference | `未通过-待-render-owner-修复` | 2026-07-11 | Text 01 下层 exact 的真实 Cargo 编译到 Render 11 测试时在 `reflection_probe_product.rs:175` 报 E0282，未进入任何 Text 断言；对应 WGPU 会话也已记录此前同文件的 E0425×2/E0282。 |

## 修复结果与回传

- 根因：The reflection-probe graph-pass assertion was accidentally placed in snapshot construction before any typed RenderStats value existed, leaving the pass closure without a concrete element type.
- 架构修复：Moved the unchanged graph-pass assertion to capture_with_framework immediately after query_stats, where last_graph_executed_passes is the authoritative typed Vec<String>; no pass-name alias, empty collection, cfg skip, or weaker assertion was added.
- 验证：The current lib-test compile advanced past reflection_probe_product without E0282. Its focused WGPU run is presently prevented by the separately routed Render 18 planar-filter E0432 failure, not by Render 11 type inference.
- 回传：Text 01 may resume its font exact after Render 18 returns the already-open planar-filter export failure; the Render 11 reflection-probe E0282 is repaired.
