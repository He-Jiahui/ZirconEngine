---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: rich-text-dto-shader06-source-staging-gate
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/text/adapter.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs
  - zircon_runtime_interface/src/ui/surface
tests:
  - cargo check -p zircon_runtime --locked
  - cargo test -p zircon_runtime --test runtime_environment_ibl_source_import_staging_contract --locked -- --test-threads=1
resolved_at: 2026-07-15
---


# Frameworks05：Rich Text DTO 阻断 Shader06 source staging

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行者：`shader06-m3-current-source-attestation-validation-20260715`
- 来源执行切片：M3 current-source source-import staging gate。
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：`zircon_runtime/src/ui/text/adapter.rs` 是 UI DTO 到 neutral Text DTO 的共享转换边界；Shader06 只消费该当前源码合同，不能实现上层特例。

## 失败现象与复现证据

Shader06 的受管作业先通过 IBL artifact contract（23/23），但 source-import staging 在当前
Text/graphics 编译错误前停止。Render18 的随后重测把同一问题收窄为 RichTextFormat、
TextDirection、TextFrame、TextWritingMode、TextAlign 与 TextWrap 的 16 个 `E0277`
双向 DTO conversion 缺口。Shader06 未修改业务文件，来源 gate 保持 pending。

## 最低共享层根因

现有转换驻留在 `cfg(ui)` 的 `zircon_runtime/src/ui/text/adapter.rs`，但 graphics-only
plugin 启用 `graphics` 而不启用 `ui`；因此 graphics consumer 在构建时根本看不到该边界。
当前源码的非 UI-gated `zircon_runtime/src/graphics/text_transport/mod.rs` 已承载
TextDirection、UiFrame/TextFrame、TextWritingMode、TextAlign 与 TextWrap 的双向映射，
但缺少 `RichTextFormat -> UiRichTextFormat` 的唯一逆映射，测试也只覆盖 UI→neutral。
Frameworks05 必须在该 graphics/text boundary 完成 hard cut 并以 graphics-only gate
验证 feature wiring；不得以启用 `ui`、复制 local adapter 或条件兼容层掩盖边界错误。
Text03 仅作为下游文本合同依赖。

## 架构修复验收

- Frameworks05 在 `graphics/text_transport` 补齐 `RichTextFormat -> UiRichTextFormat` 并将所有 RichTextFormat variants 的双向 round-trip 纳入该模块测试；保持唯一 canonical mapping，禁止旧类型兼容层。
- 新增受管 graphics-only gates：`cargo check -p zircon_runtime --no-default-features --features graphics --locked` 与 `cargo test -p zircon_runtime --no-default-features --features graphics --lib graphics::text_transport::tests:: --locked`。两者通过后，再以新的受管 `cargo check -p zircon_runtime --locked` 消除该组 E0277，再由 Shader06 使用新的受管 lane 重跑自己的 source-import staging contract。
- Shader06 artifact contract 的 23/23 证据保持不变；source-import gate 只有在实际开始并完成后才可更新。

## 禁止临时方案

- 不得修改 Shader06 文件、增加 Shader-local conversion、类型别名、facade、启用 `ui` 的 `cfg` 回退或测试专用转换。
- 不得复用旧二进制或历史成功输出替代 fresh managed gate。
- 不得在失败的 Frameworks05 check 后 freeze manifest、标记验证成功或 closeout。

## 修复结果与回传

- 根因：The Text hard cut exposed Ui-versus-neutral DTO drift and stale SDF/UI lib-test consumers before Shader06 source-staging tests could compile.
- 架构修复：Converged neutral transport ownership outside the ui feature gate, migrated all Text-owned consumers and test fixtures, and restored default plus graphics-only lib-test compilation.
- 验证：Default current lib-test focused gates passed 9/9 measure, 3/3 shaped-cache, 8/8 hit-testing, 6/6 rich-format; boundary 12/12; target-server exit 0; graphics-only plugin 9 passed, 0 failed, 2 ignored.
- 回传：Shader06 can rerun its blocked environment-IBL contract on a Text-compile-consistent source boundary; its separate environment IBL assertion remains Shader06-owned.
