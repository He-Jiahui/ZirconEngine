---
handoff_kind: failure
status: open
created_at: 2026-08-16
summary_slug: text-retained-layout-split-imports
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/zircon_runtime/text/02
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/runtime_lines.rs
tests:
  - ".\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -RepoRoot E:\\Git\\ZirconEngine -Package zircon_app -Bin zircon_editor -NoDefaultFeatures -Features target-editor-host -SkipTest"
---

# Text 02: retained text layout split imports block the Editor product build

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source Editor build and native WGPU visual acceptance
- 修复责任计划：`docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md`
- 交接原因：失败位于 Text02 明确声明 write scope 的 retained-host glyph artifact/layout 拆分，不属于 UI12 的 device-pixel AA 或 WGPU rounded primitive ownership。

## 失败现象与复现证据

受管 Cargo job `e3983bf8dd0145289b25fee93530ca26` 于 2026-08-16 03:25-03:37 构建 `zircon_app --bin zircon_editor --no-default-features --features target-editor-host`。Runtime 与 WGPU 已编译完成，`zircon_editor` 最终报告 20 条错误；其中 10 条由下列两个 Text02 文件产生：

- `layout/runtime_lines.rs:7`：`super::super::super::data::FrameRect` 少一层祖先，当前层级需到 `host_contract::data`。
- `layout/runtime_lines.rs:8`：`super::super::font` 少一层祖先，当前层级需到 `paint_text::font`。
- `layout/runtime_lines.rs:9`：`runtime_text_layout_frame` 不在 `layout::metrics`，实际 owner 是 `paint_text/draw/metrics.rs`；`empty_runtime_line_frame_x` 仍在 `layout::metrics`，导入应按 owner 拆开。
- `layout.rs:269,337,392,418,439,456,726`：函数签名继续使用 `ShapedGlyph`，但文件级 `zircon_runtime::text::ShapedGlyph` 导入在拆出 `runtime_lines.rs` 时被一并移走。

结构化 rustc 诊断来自：

`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d\debug\.fingerprint\zircon_editor-97202e8491a8008a\output-lib-zircon_editor`

## 最低共享层根因

Text layout 的新 folder-backed split 改变了 `runtime_lines.rs` 的模块深度，但相对导入仍沿用拆分前层级；同时 `layout.rs` 仍拥有 shaped-glyph 对齐与宽度函数，拆分时删除了它自身需要的 `ShapedGlyph` 导入。错误均为 split-owned visibility/import regression，不应由 UI12 复制 text 类型或增加 facade 绕过。

## 架构修复验收

- `runtime_lines.rs` 从真实 owner 模块导入 `FrameRect`、font helpers、layout-local metrics 与 draw metrics，不增加新的跨层 re-export。
- `layout.rs` 显式导入其七处签名使用的 `ShapedGlyph`。
- scoped rustfmt 与 diff check 通过。
- 上述受管 `zircon_editor` 产品构建中这 10 条诊断归零；不能只跑不编译 `zircon_editor` 的 Runtime framebuffer 测试。

## 修复结果与回传

Open state: `待 Text02 owner 修复`; UI12 不宣称 Editor 产品构建或视觉验收通过。
