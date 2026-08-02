---
plan: zircon-editor-11
milestone: M2.1a
status: superseded
session: editor11-appearance-preferences-version-shell-m2-20260718
related_code:
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/settings/io.rs
tests:
  - zircon_editor/src/core/settings/tests.rs
---

# Appearance Preferences Unified Version Shell Hard Cut

> Superseded (2026-08-02): the private `ui/preferences` persistence shell described below was retired. The canonical `zircon.editor.settings` document and typed appearance tokens now belong to `core/settings/`, which rejects legacy and v0 payloads. The obsolete Python source receipt and its private TOML fixture were deleted; this child record is historical evidence, not replay guidance or a current validation entry point.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-07-18 17:22 +08:00 | `implemented-validation-pending` | `EditorAppearancePreferencesDocument` 已实现 `VersionedSchema`，schema id 固定为 `zircon.editor.appearance-preferences`、当前版本为 1；新 writer 统一输出 canonical JSON `$zircon` envelope，payload 内嵌 `version`、`APPEARANCE_PREFERENCES_VERSION` 和 appearance DTO 转换 helper 已硬删除。旧 v1/v2 TOML 只投影为无壳 v0 并经过 `MigrationStep(0)`，v1 typography 继续精确迁移旧默认字号，v2 保持当前逻辑像素；不保留 TOML writer、双写或兼容 facade。load 结果显式携带 `migrated_from`，startup 记录需要重存；future shell fail closed。新增真实 v1 fixture 与 current round-trip、迁移后 canonical resave 幂等、future 拒绝、path metadata Rust 回归。 | TDD RED：实现前 Python 合同 4/4 失败。GREEN：`python tools/tests/test_editor11_appearance_preferences_version_shell_contract.py` 4/4；`rustfmt --edition 2021 --check` 覆盖 4 个精确 Rust 文件并通过。第一次 GREEN 运行仅暴露静态测试对 rustfmt 换行的错误耦合，改为跨空白语义正则后通过，生产行为未变。Cargo 未启动：Coordinator01 当前要求 immutable full compile-input snapshot，且 Editor10/Frameworks01/Render02 有既定依赖门禁；本记录不伪报 Rust 编译或运行时通过。M2.1 的 keymap、journal、layout 三面仍开放，因此父里程碑不勾选。 |
| 2026-08-02 | `superseded` | 当前架构已硬切到 `core/settings` 的统一 `zircon.editor.settings` v1 文档；旧 `zircon.editor.appearance-preferences` reader、TOML v0 迁移、私有 fixture 与源码字符串 receipt 均不再是产品合同。 | current-source 审阅确认 `core/settings/io.rs` 对 legacy/v0 fail closed，现有 `core/settings/tests.rs` 覆盖 typed appearance tokens、canonical round-trip、旧格式/旧 schema 拒绝与原子应用；旧 Python receipt 实跑 0/4，均因退役 owner 不存在而报 `FileNotFoundError`。未运行 Cargo，不把静态审阅冒充 Rust GREEN。 |
