---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: glyph-atlas-storage-format-import
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/text/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/text/atlas/mod.rs
tests:
  - cargo build -p zircon_app --bin zircon_shader_pbr_viewer --locked --release
---

# Text01: glyph atlas storage format import

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：EC-M5 current-source Release viewer gate
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 交接原因：`scene_renderer/ui/text.rs` 与 `src/text` 的 glyph-atlas/public text contract 均在 Text01 的已注册 write scope；Shader06 不应在对方进行中的 text runtime 重构中补写导入。

## 失败现象与复现证据

Shader06 以 Windows/MSVC、`--locked`、coordinator external target 运行：

```text
cargo build -p zircon_app --bin zircon_shader_pbr_viewer --locked --release
```

FIFO reservation `dad3b8f4987445b59c022f728615ffe9` 已消费为 job `7bf1197f62d942eab53d8d2228c791e6`，run `aeb263813ed243428a20b30e82a61c28`。该 run 在 `zircon_runtime` 编译阶段以 `E0433` 终止：

```text
zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs:357:36
use of undeclared type `GlyphAtlasStorageFormat`
help: use crate::text::atlas::GlyphAtlasStorageFormat;
```

预期是 current-source Release viewer 能完成链接，以便继续 `--help`、Release capture-flag 拒绝和 DX12 Ready-frame 验收。

## 最低共享层根因

当前未提交的 Text01-owned `ui/text.rs` 重构移除了 `use crate::text::atlas::GlyphAtlasStorageFormat;`，但 `SingleStorageReplacement` 分支仍以该枚举作为 `unwrap_or(...)` 的 fallback。该层是 glyph atlas storage contract 的消费者，不能由 Shader06 以局部别名、默认值替换或跳过 native atlas 分支掩盖。

## 架构修复验收

- 在 Text01 的 canonical glyph-atlas consumer 边界恢复/收敛 `GlyphAtlasStorageFormat` 的唯一合法引用，并保持 native bitmap atlas 的 storage fallback 语义。
- 使用 Text01 的 focused native bitmap atlas/text consumer gate 验证该模块；不得只运行格式化或静态扫描。
- 重新运行来源 Release 命令，确认 `zircon_shader_pbr_viewer` Release binary 完成链接；随后 Shader06 恢复其 CLI 和 DX12 upward gate。

## 禁止临时方案

- 不得在 Shader06 添加别名、兼容 shim、call-site special case 或 test-only fallback。
- 不得删除 `unwrap_or(GlyphAtlasStorageFormat::R8Unorm)`、跳过 `SingleStorageReplacement` 或弱化 Release 验收以隐藏编译错误。

## 修复结果与回传

Current-source recovery (2026-07-29): `ui/text.rs` no longer reads
`GlyphAtlasStorageFormat`. The former `SingleStorageReplacement` fallback now
uses `bitmap_frame.atlas_format()` directly; an absent format prepares the
bitmap renderer idle and takes the existing glyphon fail-closed path. Mixed
storage submissions likewise forward their explicit `atlas_format`. This
removes the stale undeclared-type reference without restoring an unused import
or weakening the fallback behavior.

Status: `implemented_validation_pending`. The focused no-default lib-test
attempt completed before text assertions because unrelated `cfg(test)` scene
and asset-worker modules failed to compile. A managed default-feature library
check and the original Shader06 Release viewer command remain required before
this record can be marked fixed.
