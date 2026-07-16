---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: text-hard-cut-runtime-consumer-type-drift
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/ui/text/adapter.rs
  - zircon_runtime/src/ui/surface/text_shape.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_advances.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/decorations.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild
  - cargo test -p zircon_runtime --test virtual_geometry_debug_snapshot_contract --locked
resolved_at: 2026-07-15
---


# Frameworks 05：Text hard-cut 后 Runtime consumer 类型边界未迁移完整

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Runtime04 project-TOML consumer 修复已完成静态门和独立复核，但 Windows 受管 full package job `8c2266701b6f49079719471e932f6fd7` 在 Text lib-test consumers 报 75 个 E0308/E0603；随后 focused VG integration job `b4bff1bc67fe4e7d962b54a91fcc53f6` 在普通 library consumers 报 44 个 E0308/E0603，均未到达 VG 测试。原 10 个 `to_toml_string()` E0599 未再出现；本 failure 不归 Runtime04。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 来源执行切片：Virtual Geometry debug snapshot project-TOML consumer Failure 修复的上行测试阶段
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：错误发生在 Frameworks05 M3 将文本物理 owner 硬切到 `zircon_runtime::text` 后的 UI/graphics consumer 适配边界；Runtime04 资产 fixture 不拥有文本 DTO、adapter 可见性或 scene-renderer UI 调用链。

## 失败现象与复现证据

2026-07-15 Windows 默认受管池首先执行：

```powershell
./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild
```

job `8c2266701b6f49079719471e932f6fd7` 退出 101，`zircon_runtime` lib-test 编译报告 75 个错误。随后同一兼容池执行 focused integration target：

```powershell
cargo test -p zircon_runtime --test virtual_geometry_debug_snapshot_contract --locked
```

该命令由协调器 job `b4bff1bc67fe4e7d962b54a91fcc53f6` 完整 acquire/start/finish/release，退出 101；普通 library 编译仍有 44 个错误，未进入 VG test binary。代表性失败：

- `ui/surface/text_shape.rs` 直接调用私有 `ui::text::adapter::text_style`，E0603；
- `TextShapeRequest::{horizontal,vertical}` 要求 `TextStyle/TextDirection/TextRange`，graphics consumers 仍传 `UiResolvedStyle/UiTextDirection/UiTextRange`，E0308；
- rich-text parser 要求 `RichTextFormat`，调用方仍传 `UiRichTextFormat`；
- decoration owner 要求 `TextFrame/TextWritingMode`，调用方与顶点 owner 仍在 `UiFrame/UiTextWritingMode` 两侧交错；
- bidi、glyph range 与 SDF fallback 返回的 Text DTO 未在 UI 边界转换回对应 Ui DTO。

Frameworks05 计划当前写为 M3 `completed`，但 fresh production/library 与 lib-test 两条门均证明 consumer migration 尚未闭合。

## 最低共享层根因

文本实现物理 owner 已从退休的 `graphics/text`、`framework/render/text` 移到 `zircon_runtime::text`，但 Text DTO 与 `zircon_runtime_interface::ui::surface` DTO 的转换 owner/可见性没有在硬切时完整接线。部分调用点直接越过私有 adapter，另一些调用点把 Ui DTO 直接交给 Text 内部 API，导致 production 与 `cfg(test)` consumers 同时失配。

## 架构修复验收

- 在 UI↔Text 边界建立窄、明确、可复用的转换入口；`ui::text::adapter` 保持实现细节，不以公开整个模块修复 E0603。
- `TextShapeRequest`、rich-text、decoration、bidi、glyph range 与 SDF fallback consumers 全部通过同一转换 owner 对齐 Text/Ui DTO；不得在 44 个报错点散布无归属的临时特判。
- retired `graphics/text`、`framework/render/text` 保持物理删除，旧 owner 扫描继续为 0。
- 重跑 focused `virtual_geometry_debug_snapshot_contract`，确认能编译并运行原有断言；再跑 `zircon_runtime -SkipBuild` package gate，production/lib-test consumer 编译错误清零。
- 更新 Frameworks05 M3 状态与编号产出记录，只有上述门通过后才能继续声明 `completed`。

## 禁止临时方案

- 不得把 `ui::text::adapter` 整模块改为公开 API，或新增 facade/re-export 绕过 owner。
- 不得恢复 `graphics/text`、`framework/render/text`、兼容 alias、双轨 DTO 或默认转换 fallback。
- 不得按编译器提示在各调用点随意堆叠 `.into()`；转换必须落在可审计的边界 owner。
- 不得删除/忽略 Runtime04 VG 测试或把 Text 编译失败计入资产模块通过。

## 修复结果与回传

- 根因：The Text physical-owner hard cut left Ui DTO consumers and feature-gated font assets inconsistent across default, graphics-only, and target-server compilation graphs.
- 架构修复：Established graphics/text_transport as the unique non-UI-gated conversion owner, migrated Text/UI/SDF consumers, gated UI-only tests, and cfg-gated CompositeFontDescriptor fields when text is disabled.
- 验证：Default current lib-test compiled and focused gates passed 9/9 measure, 3/3 shaped-cache, 8/8 hit-testing, 6/6 rich-format; target-server job c8839193c97a4c0d9993d4a81e8268fe exited 0; graphics-only plugin job 805493d44b784f7486cbf9daf7c53a77 passed 9/0/2.
- 回传：Frameworks05 Text consumer migration is compile-consistent; Runtime04 can rerun its own asset/VG contracts without Text import/type blockers.
