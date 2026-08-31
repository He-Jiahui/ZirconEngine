---
related_code:
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/layout/main_host_page_layout.rs
  - zircon_editor/src/ui/workbench/layout/manager
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/host/layout_hosts/repair_builtin_shell_layout.rs
related_tests:
  - tools/tests/test_editor06_workbench_layout_single_source.py
  - zircon_editor/src/tests/workbench/layout/layout_preset_persistence.rs
  - zircon_editor/src/tests/workbench/layout/window_drawer_ownership.rs
plan_sources:
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
status: in_progress
---

# Editor06 Workbench layout single-source hard cut

## 架构裁决

`WorkbenchLayout.activity_windows` 是 Workbench 窗口布局的唯一持久化状态树。Drawer、区域约束和
View 约束仅由对应 `ActivityWindowLayout` 持有；根级 `drawers`、`region_overrides`、
`view_overrides` 以及根/窗口同步函数全部删除，不保留 serde 默认回退、双写、alias 或兼容迁移。
Workbench 主页面必须显式声明 `activity_window`，未知旧字段由 `serde(deny_unknown_fields)` 拒绝。

参考 Unreal Engine `FTabManager::FLayout` 的单一 `Areas` 布局树，以及 `PersistLayout()` 从 live
docking areas 直接收集、`RestoreFrom()` 直接消费同一树的所有权模型。本切片只做结构正确性硬切；
未在缺少 profile 的情况下开展性能优化，也不声明未经测量的耗时或功耗收益。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据与后续 |
|---|---|---|---|
| 2026-08-29 03:42 +08:00 | `implementation-complete / static-contract-green / managed-rust-pending` | 删除 `WorkbenchLayout` 三组根级镜像字段和两条 legacy 合成/同步路径；布局命令、normalize/restore、preset capture/restore、builtin shell repair、autolayout、committed shell state、retained host 与相关测试夹具全部改为窗口级 owner；默认 JSON fixture 将 drawer 嵌入 `window:workbench`，Workbench 页显式绑定 owner。 | `test_editor06_workbench_layout_single_source.py` 4/4，10.780s；默认 fixture JSON 解析通过；全编辑器旧同步符号命中 0；全 `WorkbenchLayout` 字面量旧根字段命中 0；定向 `rustfmt` 通过。Windows 受管 `zircon_editor` check 未进入 Cargo，协调器返回 `cargo_reuse_pool_busy`（占用作业 `8b31357dce8a42f89e0c51212d3774fc`）；不轮询、不启动旁路 Cargo。待后续受管 compile 与 focused tests 通过前保持 `in_progress`，不提交、不发送企微。 |
| 2026-08-29 14:45 +08:00 | `implementation-complete / static-contract-green / managed-rust-pending` | 补齐 bootstrap 全局布局修复、默认预览 fixture、drawer attachment、document workspace 和 roundtrip/restore 测试中的根级 `layout.drawers` 遗漏；只读路径统一走 `active_activity_window_drawers()`，可变路径显式取得 `active_activity_window_mut().activity_drawers`；源码契约新增 Workbench 语义接收者旧根字段访问扫描。 | 两组 Editor06 源码契约合计 9/9，38.745s；5 个相关 Rust 文件定向 `rustfmt --check` 通过；`BottomLeft/BottomRight` 枚举引用、旧同步符号、疑似 Workbench 根级旧字段访问均为 0 命中；快照层 `WorkbenchSnapshot.drawers` 保持不变。Windows 受管编译仍沿用上条阻塞，不轮询、不旁路。 |
| 2026-08-29 17:48 +08:00 | `implementation-complete / static-contract-green / managed-rust-pending` | 完成剩余 WorkbenchLayout 根访问复核，补齐 roundtrip/restore 与 4 个测试 owner 的窗口级访问；布局命令的 drawer 校验后访问由 3 处生产 `expect` 硬切为 typed `LayoutCommandError::MissingDrawer`；`focus_instance` 删除空 activity-window 表上的无效默认窗口调用，目标 drawer 在任何 mutation 前安全取得，生产路径不再 panic 或静默合成 owner；`ActivateMainPage` 新增 `MissingMainPage`、`DuplicateMainPage` 与 `MissingActivityWindow` 三级 typed preflight，页面身份或其窗口 owner 无效时保持完整 layout 不变；`ResetToDefault` 仅在默认布局确有差异时替换和发布 changed；split ratio 与 drawer extent 的非有限输入分别由 `NonFiniteSplitRatio`、`NonFiniteDrawerExtent` 在首次可变目标访问前拒绝；为落实结构规范，`apply.rs` 内 425 行混合测试已硬切到 folder-backed `apply/tests/{source_guards,no_op,drawer_commands}.rs` 三个语义 owner，根测试模块只保留 3 行挂载，未保留旧测试 wrapper；新增 800 行 owner budget 回归守卫；未改变 `WorkbenchSnapshot` 投影所有权。 | Editor06 合同仍为 9/9，其中 single-source 最新 5/5（18.003s）；missing-page/window、duplicate-page、repeated-reset 与 non-finite geometry 四组 no-op Rust 行为测试已落盘；11 个迁移测试加 1 个结构预算守卫共 12 项，生产 `apply.rs` 由 758 降至 336 行，测试 owner 为 3/39/211/201 行，全部低于 800 行门；5/5 模块路径解析、定向 `rustfmt --check`、typed preflight/no-op 顺序检查与 scoped `git diff --check` 通过，生产 `apply/focus` 的 `.expect(` 均为 0。复核另确认 `MainHostPageLayout.document_workspace` 与 `ActivityWindowLayout.content_workspace` 仍形成 primary-tabs 双 owner，且结构命令仍有 detach-before-target-validation P0；二者必须按既有 Optimize 的单一 LayoutAuthority、原子事务与位置索引计划硬切，不以整树 clone 或局部 preflight 代替。受管 Rust 行为门仍待协调器执行，不提交、不发送企微。 |
