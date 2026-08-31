---
plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
child_plan: docs/plans/zircon_editor/editor/08
status: source_complete_validation_pending
date: 2026-08-29
---

# Editor08 Command Localization Projection Hard Cut

## 目标

将命令 identity、执行路由、菜单结构和 locale-bound presentation 分离，形成与 Unreal `FUICommandInfo` 相同的稳定 command identity/context/chord 边界，同时保留 Zircon 的 bundle ticket 生命周期和热 locale 投影能力。

## 架构产出

- `EditorCommandPresentation` 保存稳定 `EditorLocalizationKey`、来源 (`Builtin` 或插件 bundle) 和 description key；生产 descriptor 不再有 `display_name`/`description`/字符串 `menu_path`。
- `EditorCommandMenuPath` 固定为 `root + groups + leaf`，segment ID 经过 lowercase identifier 校验；菜单排序、分组和去重只使用稳定 ID。
- `EditorMenuItemDescriptor` 序列化只输出类型化 `menu_path`；反序列化从 segment ID 重建 `stable_path` 缓存并以 `deny_unknown_fields` 拒绝旧字符串路径/shortcut 载荷，不能注入与类型化路径不一致的缓存值。
- `EditorCommandPaletteCatalog` 只保存中性 seed、enablement 与 canonical search metadata；locale projection 是有界容量 4 的不可变缓存，不复制或修改 registry。
- 菜单和 retained palette 每次从 `EditorI18nService` 捕获 locale；locale 变更只重建 presentation projection，command ID、MRU、when、keymap 和 factory 保持不变。
- serialized plugin command ABI 升级为 `zircon.editor.command/2`，菜单 ABI 升级为 `zircon.editor.menu/2`；command 声明 bundle ID/label key/description key，menu 声明稳定 root/group/leaf ID 与各 segment localization key。`into_contribution_batch` 原子校验并绑定 bundle，缺 bundle、缺 key 或 group ID/key 数量不一致均拒绝注册。
- 第一方命令菜单由 `EditorCommandDescriptor.menu_path` 单一投影，插件不再并行注册同一批静态 `menu_items`；动态 toolkit/serialized extension 菜单仍通过类型化 descriptor 合流。

## 参考与调研结论

UE `FUICommandInfo` 将 command identity、context、input chords 与 localized `FText` 分离；Zircon 采用同一边界，并把插件本地化 bundle 绑定作为生命周期 owner。palette 的热路径继续采用 rarest posting + bounded query window，避免每次输入构造全量 display row vector。

## 验证计划

- 静态：精确 Rust 2021 rustfmt、scoped diff check、旧 descriptor constructor/display/menu API 扫描。
- 受管：`zircon_editor` command/registry/menu/palette 目标 Cargo 编译和行为回归，覆盖 en/zh-CN 菜单、palette、bundle raw-key、locale cache、稳定分组顺序。
- 性能：沿用 1k/10k command palette 与 menu single-pass fixture，记录 p50/p95、visited entries、owned buffers、projection cache 上限；只在 managed validation receipt 产生后填写数值。
- 独立复核：确认当前 EditorLayout07 `shell_projection.rs` successor 完成旧构造器迁移后，再执行 C/I/M review 与 Failure return。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-29 | descriptor/menu/palette/plugin ABI source hard-cut | `completed` | typed presentation/menu modules、bounded locale projection、SDK/serialized command/2 + menu/2 fixtures、retained/workbench/runtime consumers、第一方插件 typed menu paths 与静态菜单单一投影均已更新；`EditorMenuItemDescriptor` 只序列化 typed path 并重建 stable cache。 |
| 2026-08-29 | 第一方命令本地化目录覆盖 | `completed` | en/zh-CN 各 532 个唯一 key，重复 `0`、locale key-set difference `0`；首批 111 个真实插件 command ID 的 label/description 共 444 个必需命中缺失 `0`，26 个 authoring view command 的 104 个必需命中缺失 `0`。 |
| 2026-08-29 | source static gates | `completed` | touched Rust files `rustfmt --edition 2021` pass；scoped `git diff --check` pass；生产 `zircon.editor.menu/1`、字符串 `with_menu_path`、字符串 `EditorMenuItemDescriptor::new` 扫描为 `0`。Frameworks01 所有的 `core/extension/toolkit/registry.rs` 尚保留一次 stable path 字符串复验，不是第二 display identity owner。 |
| 2026-08-29 | managed Cargo validation and independent review | `pending` | 当前 EditorLayout07 owner 的 `zircon_editor/src/tests/workbench/view_model/shell_projection.rs` 仍有 2 个字符串 menu descriptor + 2 个双参数 command constructor；跨 owner 前向迁移完成前不声明 Cargo、C/I/M、Failure return、commit 或性能数值。 |
