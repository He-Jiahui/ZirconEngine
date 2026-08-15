---
handoff_kind: failure
status: open
created_at: 2026-08-05
updated_at: 2026-08-05
summary_slug: plugin-settings-page-localization-contract
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/12
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/editor_contribution.rs
  - zircon_editor/src/core/plugin/materializer.rs
  - zircon_editor/src/core/settings/page.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/extension/store/batch.rs
tests:
  - python -m unittest tools.tests.test_editor12_plugin_settings_page_localization_contract -v
  - cargo test -p zircon_runtime_interface --locked
  - cargo test -p zircon_editor --lib core::plugin --locked --jobs 1 -- --test-threads=1
---

# Editor12: 插件 SettingsPage 仍保留 raw display/category 的 V1 契约

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M1 SettingsPresentation hard cut 与 M3 plugin locale-resource 接线。
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：序列化 contribution DTO、cdylib materializer 与插件包词条声明属于 Editor12；Editor17 不得在 `SettingsPageDescriptor` 外层保留 raw 文案兼容层，或反向依赖插件 manifest。

## 失败现象与复现证据

当前 `zircon_runtime_interface/src/editor_contribution.rs` 的
`SerializedEditorContribution::SettingsPage` 仍使用 schema
`zircon.editor.settings-page/1`，并序列化 `display_name: String` 与
`category_path: String`。`core/plugin/materializer.rs` 将它们原样传入
`SettingsPageDescriptor::new`；该 descriptor 继续持有 raw 字符串，
`EditorExtensionRegistry` 和 `ContributionBatch` 以 slash-separated path 验证。

这与 Editor17 已完成的 settings presentation hard cut 相冲突：builtin setting 的
label、description、category 只能由已验证的 locale-neutral key 表示，翻译仅在消费边界发生。插件设置页若继续把英文文案和 `Plugins/Foo`
路径作为 authority，则语言热切换、插件资源包与 cdylib/rlib 等价无法收敛到同一数据源。

## 最低共享层根因

插件 contribution DTO 在 SettingsPage 变体上仍冻结旧 raw UI DTO，而不是可由插件包 locale
资源解析的 typed presentation key。由于 materializer 只做 raw string 透传，host registry 无从区分
locale identity 与已本地化文案，也无法拒绝过期 V1 payload。

## 架构修复验收

- 将 SettingsPage contribution 硬切到新的、明确版本化的 locale-key schema：title、description 和每级 category 都是受验证的 key；category 为离散 key 序列，不再是 slash-separated string。
- 旧 `zircon.editor.settings-page/1` 及 `display_name`/`category_path` raw payload 必须拒绝反序列化或 schema 验证；不得提供 alias、默认翻译、raw fallback 或 V1-to-V2 迁移分支。
- materializer 只能构造 typed settings-page presentation，并依据插件 package 的 locale-resource 声明验证 key 的可解析归属；host 的 builtin `settings.*` namespace 不得被错误地当作插件词条的唯一 namespace。
- extension registry/store 的 settings-page validation 应验证 typed key/path 结构，保留 ID 去重与原子批次语义；不能恢复 slash path 检查来通过旧 fixture。
- runtime-interface DTO roundtrip、V1 rejection、cdylib materialization、插件 locale-resource 缺失/未知 key rejection，以及 rlib/cdylib 最终 descriptor 等价均有定向回归。Editor17 的设置页呈现只消费 typed descriptor 并在 locale snapshot 边界翻译。

## 禁止临时方案

- 不得保留 `display_name: String`、`category_path: String`、`is_valid_category_path` 或以其作为 V2 的兼容辅助字段。
- 不得把插件页面文案复制进 editor core 的内嵌 en/zh-CN bundle，或在 materializer 中合成英文默认值。
- 不得仅更改 schema 字符串而继续接受 V1 字段，或将旧 path 拆分后伪装为 locale key。

## 修复结果与回传

Open state：`source_contract_drift_recorded / no_local_rollback / target_validation_pending`。
本记录只固定当前架构漂移和跨计划 owner；未运行 Cargo、未修改 Plugin/runtime-interface
源码，也未宣称 V2 API 已实现。Editor17 继续保持其已集成的 settings presentation hard cut，不能为
旧插件 SettingsPage DTO 回滚或增设兼容入口。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-05 | Editor17 M1 -> Editor12 SettingsPage locale contract handoff | `open_handoff_recorded` | 直接检查 `editor_contribution.rs` 的 V1 raw fields、`plugin/materializer.rs` 的 raw 透传、`settings/page.rs` 的 raw storage 以及 registry/store 的 slash-path validation；它们与已落地 builtin `SettingsPresentation` key-only authority 矛盾。已明确 V2 hard-cut、plugin locale-resource 验证和双轨等价验收；没有回滚主干或修改目标 owner 文件。 |
