---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: rich-text-dto-render18-retest-gate
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/text_transport/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs
  - zircon_runtime_interface/src/ui/surface
tests:
  - cargo check -p zircon_runtime --no-default-features --features graphics --locked
  - cargo test -p zircon_runtime --no-default-features --features graphics render_volumetric --locked
resolved_at: 2026-07-15
---


# Frameworks05：Rich Text DTO 阻断 Render18 fresh retest

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行者：`render18-af-m2-rebase-20260715`
- 来源执行切片：AF-M2 rebase 后的 fresh managed runtime retest。
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：Render18 的 16 个 `E0277` 发生在 UI↔neutral Text DTO adapter 共享边界；advanced-lighting 计划不拥有该类型合同。

## 失败现象与复现证据

Render18 retest `ffcc6c2e3f914f3fa024962764760b68` 将先前的 Text blocker 收窄为 16 个
缺失的 bidirectional DTO conversion：`RichTextFormat`、`TextDirection`、`TextFrame`、
`TextWritingMode`、`TextAlign` 与 `TextWrap`。Render18 与 Shader06 均拒绝添加上层
special case；该受管 retest 不能作为 accepted/frozen evidence。

在 Frameworks05 已把转换移动到 `graphics/text_transport` 后，新的受管 Render18
graphics-only 作业 `b34eeebc14034295b0bd2ece23c0624f` 已成功编译 production
`zircon_runtime` library，随后才以 exit 101 停在**下游测试 cfg/fixture 漂移**，尚未执行
`render_volumetric` 测试。具体是 `text_advances.rs` 未处理 `Result<Vec<_>>` 的 call site/
field access（`E0308`/`E0609`/`E0599`）、`ui/text/tests.rs` fixture 漏填
`layout_fallbacks`（`E0063`），以及 `runtime_text_multilingual_product_framebuffer`
integration 在 graphics-only 图中错误假设 `ui` feature 已启用。Render18 未修改这些路径，
暂存仍为 0。

## 最低共享层根因

现有 `zircon_runtime/src/ui/text/adapter.rs` 转换受 `cfg(ui)` 限制，而 graphics-only
plugin 仅启用 `graphics`。当前源码已将 canonical mapping 收敛到 non-UI-gated
`graphics/text_transport`，且本次 production-lib 成功证明该转换已进入 graphics graph。
剩余根因转为 Text/UI 测试层的 feature ownership：仅 UI 可用的测试/integration 没有正确
`cfg(feature = "ui")` 门控，同时 Result API 与必填 fixture 字段的消费者未同步合同。它们
必须在共享 Text/UI 测试边界修复，不能让 Render18 启用 `ui` 或删除 graphics-only 覆盖。

## 架构修复验收

- Frameworks05 保持 `graphics/text_transport` 的唯一 canonical conversion；Text/UI owner 必须将 UI-only tests/integration 正确 gate 在 `feature = "ui"`，更新 `Result<Vec<_>>` 消费者与 `layout_fallbacks` fixture，且不得删除 graphics-only tests。
- 必须先执行新的 managed `cargo check -p zircon_runtime --no-default-features --features graphics --locked`，再执行新的 managed `cargo test -p zircon_runtime --no-default-features --features graphics render_volumetric --locked`；两者成功并确实进入 Render18 目标测试后，才可回传 Render18 AF-M3 重跑。
- Frameworks05、Render18 在 fresh check/retest 未通过前均不得 freeze、accepted 或 milestone closeout。

## 禁止临时方案

- 不得修改 Render18/Shader06 业务路径、添加 local adapter、alias、facade、为 graphics gate 启用 `ui` 的回退或删除/绕过测试。
- 不得复用 `ffcc6c2e3f914f3fa024962764760b68` 前的任何成功输出作为新 gate。
- 不得手动暂存、提交、释放或删除其它 Session 的受管作业。

## 修复结果与回传

- 根因：Ui DTO conversions were owned below cfg(feature=ui), so graphics-only consumers could not reach them; lib-test SDF consumers also retained stale helper signatures and UI-only fixtures.
- 架构修复：Moved canonical Ui-to-neutral DTO mappings to private graphics/text_transport, kept ui adapter thin, gated UI-only tests, converged Result and SDF consumers, and removed unconditional paragraph indent shaping.
- 验证：Boundary guard 12/12; default lib-test measure 9/9, shaped-cache 3/3, hit-testing 8/8, rich-format 6/6; target-server check exit 0; current graphics-only volumetric plugin job 805493d44b784f7486cbf9daf7c53a77 ran 9 passed, 0 failed, 2 ignored.
- 回传：Frameworks05 Text hard-cut consumers are compile-consistent and the original Render18 graphics-only upward gate now executes and passes.
