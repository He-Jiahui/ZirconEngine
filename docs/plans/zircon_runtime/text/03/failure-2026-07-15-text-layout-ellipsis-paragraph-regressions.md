---
handoff_kind: failure
status: open
created_at: 2026-07-15
summary_slug: text-layout-ellipsis-paragraph-regressions
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
fixing_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/zircon_runtime/text/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/text/layout_engine/ellipsis.rs
  - zircon_runtime/src/ui/text/layout_engine/paragraph_layout.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_inline.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/overflow.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/rich_inline.rs
tests:
  - cargo test -p zircon_runtime --lib ui::text::layout_engine::tests:: --locked
---

# Text03：省略号与段落 rich-inline 布局回归

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行切片：Frameworks05 M3 共享文本服务契约硬切后的完整 Runtime 回归门
- 修复责任计划：`docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`
- 交接原因：物理 owner 与 import 硬切已完成编译、架构守卫和原始 GPU 复现；剩余失败发生在 Text03 持有的 ellipsis projection、rich inline 与 paragraph alignment 行为语义，Frameworks05 不应通过改期望值或迁移特例接管布局算法。

## 失败现象与复现证据

2026-07-15 Windows 诊断命令执行：

```powershell
cargo test -p zircon_runtime --lib 'text::' --locked --target-dir D:\cargo-targets\zircon-engine\pool\841a130ffbd3fd2e938e76b488988119044b676acced751dae7166d95d7f1025 --color never
```

日志结果为 `670 passed / 5 failed / 7446 filtered out`。协调器审计确认原先关联的 `7511f113ee874424bd9b38e778bee00c` 实际没有命令、没有 start/exit code，且 stale PID 已由 owner 精确释放，因此该日志只作为复现红灯，不作为受管通过证据。其中一条 graphics/Shader06 失败不属于本 handoff；Text03 的四条失败为：

- `word_ellipsis_trims_partial_word_before_marker`：预期合并为 2 个 resolved runs，实际为 6。
- `html_inline_image_respects_end_ellipsis_without_placeholder_fallback`：实际保留了省略号前多余的空格 run，得到 `["a", U+FFFC, " ", "…"]`。
- `html_inline_image_respects_start_middle_and_word_ellipsis`：至少一个 case 的 resolved run 数预期 4、实际 3。
- `bbcode_paragraph_alignment_reaches_resolved_line_frames`：预期 3 行 `alpha` / `beta` / `gamma`，实际只得到 2 行。

这四个测试文件相对 `HEAD` 无内容改动；Frameworks05 本切片对相关 production 文件仅把旧 `graphics::text` / `framework::render` import owner 改为 `crate::text`，未改布局算法或断言。日志：`E:/ZirconBuilds/frameworks05-m3-text-lib-tests-20260715.log`。

## 最低共享层根因

当前能证明的最低边界是 Text03 的 UI layout projection：word ellipsis 的 source/run 合并、inline object 周围空白的 ellipsis trimming，以及 paragraph override 到 resolved line frame 的传播没有满足现有契约。具体共同算法根因尚待 Text03 owner 在 `ellipsis.rs`、`rich_inline.rs` 与 `paragraph_layout.rs` 向下诊断；不是 Frameworks05 物理模块解析或旧 namespace 兼容问题。

## 架构修复验收

- 精确运行 `cargo test -p zircon_runtime --lib ui::text::layout_engine::tests:: --locked`，四条上述回归全部通过，且不得只改断言迎合错误输出。
- 保持单一 `zircon_runtime::text` 实现 owner，不恢复 `graphics::text`、`framework::render::text` alias、re-export 或 forwarding module。
- 重跑完整 `cargo test -p zircon_runtime --locked`；若仍有外部计划失败，必须按最低 owner 单独回传，Text03 四条不得再出现。

## 禁止临时方案

- 不得增加兼容路径、测试专用 bypass、重复 layout truth、调用点特判或静默丢弃 inline object/空白。
- 不得削弱 resolved run、source range、行数或 alignment 断言来隐藏实际布局回归。

## 修复结果与回传

待 Text03 修复并完成 `failure return`；当前不声明上述四条布局测试通过。
