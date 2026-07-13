---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: font-decoration-display-size-argument
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/text/01
related_code:
  - zircon_runtime/src/graphics/text/font/decoration_metrics.rs
tests:
  - cargo test -p zircon_editor --lib --no-run --locked --jobs 1
resolved_at: 2026-07-13
---


# Text 01：字体装饰度量缺少显示尺寸参数

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1.4 source authority 与只读 command when/dispatch guard 的 Windows 编译门禁
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 交接原因：失败位于 `graphics/text/font` 装饰线度量 owner；Text01 FR-M2 明确拥有 underline/strikeout metrics，Editor09 不应在资产管理切片中修改字体缩放语义。

## 失败现象与复现证据

2026-07-13 在 Windows 受管 Cargo 目标池执行
`cargo test -p zircon_editor --lib --no-run --locked --jobs 1`，编译在到达 `zircon_editor` 前失败：

- `zircon_runtime/src/graphics/text/font/decoration_metrics.rs:68` 调用
  `Self::from_font_units(FontAssetFaceMetrics { ... })`；
- `from_font_units` 当前签名为
  `from_font_units(metrics: FontAssetFaceMetrics, display_px: f32)`；
- rustc 报 `E0061`，明确缺少第 2 个 `f32` 参数。

完整日志：`.codex/tmp/editor09-m1-4-source-authority-compile-r1-20260713.log`。

## 最低共享层根因

已证明的最低边界是字体装饰度量构造器签名与同文件字体 face 转换调用发生漂移。`display_px` 影响 underline/strikeout 的像素缩放，必须由 Text01 确认正确来源，不能由 Editor09 猜测常量或绕过。

## 架构修复验收

- Runtime 字体 face 到装饰度量的转换显式传入正确的显示像素尺寸，并以 focused 测试锁定非默认尺寸缩放语义。
- `cargo check -p zircon_runtime --lib --locked` 通过。
- 原始复现 `cargo test -p zircon_editor --lib --no-run --locked --jobs 1` 越过 Runtime 编译门禁，Editor09 M1 可恢复向上验证。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止在 Editor09 注入固定字号、跳过 Runtime 编译或复制字体装饰计算。

## 修复结果与回传

- 根因：Text01 from_font_units signature added display_px while its face-metrics caller remained on the old arity.
- 架构修复：Text01 supplied the authoritative display pixel size at the font decoration owner; no fixed-size fallback or Editor shim was added.
- 验证：cargo test -p zircon_editor --lib --no-run --locked --jobs 1 reached successful test-binary generation; artifact .codex/tmp/zircon_editor-editor09-m1-4-source-authority-r4-20260713.exe.
- 回传：Editor09 r4 no-run gate compiled past Text01 and produced the current zircon_editor lib-test binary.
