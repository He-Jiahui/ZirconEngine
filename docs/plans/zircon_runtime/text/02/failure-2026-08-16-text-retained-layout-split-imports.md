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

## 禁止临时方案

- 不得新增 facade re-export、复制 text 类型或恢复拆分前的模块布局来掩盖导入层级错误。
- 不得以静态扫描、只编译 Runtime 或跳过 `zircon_editor` 产品构建替代声明的上游验收门。

## 修复结果与回传

Open state: `implementation_complete_static_checked / managed_editor_build_pending`; UI12 不宣称 Editor 产品构建或视觉验收通过。

2026-08-26 current-source 复核确认该 split-owned 导入已在原 owner 前向闭合：
`runtime_lines.rs` 从四层祖先导入 `FrameRect`/font owner，将
`runtime_text_layout_frame` 与 layout-local `empty_runtime_line_frame_x` 分别从真实 owner 导入；
`layout.rs` 也显式持有其七处签名所需的 `zircon_runtime::text::ShapedGlyph`。没有新增 facade、
类型复制或旧布局恢复。两个 production owner 的 scoped Rustfmt 已通过；声明的 managed
`zircon_editor` 产品构建尚未执行，因此 failure 保持 `open`，只把“待代码修复”收窄为动态上行验收。

### 2026-09-19 successor static source-contract receipt

Fixing Session `failure-roll-01a084c8-text02-retained-layout-r1` sealed ticket
`20daf48919394431a69ba7b18439d927`. Coordinator copy job
`17bcfeb06930459faa4abafd5b957887` and run
`20daf48919394431a69ba7b18439d927` exited 0 with
`TEXT02_RETAINED_LAYOUT_SPLIT_IMPORT_SOURCE_CONTRACT_PASS`
(`CHECKED_PATHS=3`). The pre-receipt source manifest was
`0fbb12680a16fe48f4500ba8d69345f112bb4d688eb193de9c160d392ed233b8`.

This receipt proves only the current split-import source contract and scoped
Rustfmt. The managed Windows `zircon_editor` target-editor-host Cargo/product
build, external `E:/Git/zr_vm` admission, independent
Critical/Important/Moderate zero-finding review, canonical `failure return`,
and coordinator closeout remain pending. The append changes the document hash
after the static snapshot; any dynamic successor must reseal the current file.

### 2026-09-20 independent source review r1

Reviewer session: `review-text02-retained-layout-r1`, child of
`failure-roll-01a084c8-text02-retained-layout-r1`. The review covered both
folder-backed production owners at the current baseline and did not change their
ownership or implementation bytes.

Result: **Critical=0 / Important=0 / Moderate=0**.

- `layout.rs` keeps its `FrameRect`, font-face and layout-policy imports at the
  `paint_text::draw::layout` owner, and delegates line production to the sibling
  `runtime_lines` module. Its current `PaintTextLayout` projection consumes the
  runtime artifact DTOs without retaining the removed `ShapedGlyph`/host-layout
  fallback path, matching the current artifact-oriented tests.
- `runtime_lines.rs` resolves `FrameRect` and font helpers through the actual
  `host_contract`/`paint_text` ancestors, imports `runtime_text_layout_frame`
  from `draw::metrics`, and keeps the line-local empty-frame behavior in the
  split owner. It does not add a facade re-export or duplicate a Runtime text
  type.
- The static contract marker and scoped `git diff --check` passed. The local
  rustfmt check reports import-order differences in the current source snapshot;
  no formatter-only rewrite was made during this review.

Current reviewed hashes:

```text
zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  bf2082c2920cafd12ec16e47536ab570b08a31f48476654855dbb0b5a3c640f7
zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/runtime_lines.rs
  3f4599594ab0bacf27033b1da5cfb8720c820d8e9bc74c8fcc1575658f40ff9e
```

The managed Windows `zircon_editor` target-editor-host build, UI12 visual/upward
acceptance, canonical `fixed-*` return and closeout remain pending. Static import
evidence is not promoted to a product build result.

## 2026-09-25 current-source rolling reconciliation (retained layout r2)

Successor Session `failure-roll-01a084c8-text02-retained-layout-r2` claimed this failure
record, the Text02 plan, and both exact folder-backed layout owners. No Rust source was edited
in this continuation. The current source probe completed with marker
`TEXT02_RETAINED_LAYOUT_CURRENT_SOURCE_PASS 8 of 8`: `layout.rs` consumes the immutable
artifact raster-face DTO and no longer carries the removed `ShapedGlyph` host fallback;
`runtime_lines.rs` imports `FrameRect`, font helpers, and draw metrics from their real owners,
resolves `UiResolvedTextGlyphArtifactLine`, and does not call the retired `shape_text_line`.

The claimed current hashes are:

| Path | SHA-256 |
|---|---|
| `zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs` | `bf2082c2920cafd12ec16e47536ab570b08a31f48476654855dbb0b5a3c640f7` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/runtime_lines.rs` | `3f4599594ab0bacf27033b1da5cfb8720c820d8e9bc74c8fcc1575658f40ff9e` |

Both source files were already dirty when r2 started; their large artifact-oriented split
diffs are existing current-source work from the prior Text02 owner and are not absorbed or
rewritten here. The failure document also carried prior receipt edits; the Text02 plan remains
clean. Scoped `git diff --check` is clean. Rustfmt check remains non-green only for import
ordering in the existing source snapshot; no formatter-only rewrite was made. The authoritative
current-source manifest is coordinator snapshot `3843`.

This is static handoff evidence only. The managed Windows `zircon_editor` target-editor-host
Cargo/product build, UI12 visual/upward acceptance, external `E:\\Git\\zr_vm` admission,
independent review, canonical fixed return, closeout SHA, and WeCom result remain pending;
historical static tickets are not reused as product acceptance.

Independent review receipt (2026-09-25): the reviewer rechecked coordinator snapshot `3843`
and confirmed both current owner hashes, the `TEXT02_RETAINED_LAYOUT_CURRENT_SOURCE_PASS 8 of 8`
marker, artifact DTO split/import ownership, absence of the retired `ShapedGlyph` and
`shape_text_line` host path, clean dirty provenance, scoped `git diff --check`, and the
import-order-only rustfmt caveat. Review result is Critical `0`, Important `0`, Moderate `0`.
This receipt is carried in post-review coordinator snapshot `3844`.
