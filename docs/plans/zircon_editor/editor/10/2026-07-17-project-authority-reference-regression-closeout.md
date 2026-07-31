---
related_code:
  - zircon_editor/src/core/project/authority.rs
  - zircon_editor/src/core/project/opened_project.rs
  - zircon_editor/src/core/project/project_probe.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/ui_asset_promotion.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing
  - zircon_editor/src/ui/workbench/project/editor_project_document_load.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_save.rs
  - zircon_editor/src/ui/workbench/project/layout_preset_assets.rs
  - zircon_editor/src/ui/workbench/startup/editor_startup_session_document_welcome_pane_snapshot.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
tests:
  - zircon_editor/src/core/project/tests/boundary.rs
  - zircon_editor/src/core/project/tests/template_creation.rs
  - zircon_editor/src/tests/host/asset_references.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - zircon_editor/src/tests/host/manager/project_generation_projection.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
plan_sources:
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/10/failure-2026-07-12-project-asset-reference-full-gate-regressions.md
  - docs/plans/zircon_editor/editor/10/failure-2026-07-17-project-open-repeated-manager-scan.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-07-18-project-source-index-targeted-import.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
doc_type: output-record
status: implementation_complete_static_green_review_recheck_and_managed_validation_pending
---

# Editor10 ProjectAuthority / AssetRef 当前源码回归收束

本记录只覆盖 Editor10 当前 P0：修正 ProjectAuthority 边界守卫的自命中，建立一次打开只构造一份 canonical `ProjectManager` generation 的宿主链，并消除 welcome、layout preset 与 UI asset locator 正常投影中的重复工程打开。sprite-atlas paint-time I/O 仍由同计划的独立 failure 记录继续处理，不在本切片假关闭。

## 架构结果

- `OpenedProject` 直接拥有已解析 manifest/registry index 的 prepared `ProjectManager` 与 summary；Runtime `AssetManager::open_prepared_project` 在该实例上完成唯一一次 source inventory/import scan，Editor asset manager、workspace watcher、工程文档 load/save 和 locator 解析随后复用同一 generation。
- 工程文档 load/save 只接受 `&ProjectManager`。退休的 `load_from_path/save_to_path` API 与 `_from_path.rs` 文件已硬删除，测试 fixture 也必须先通过 `ProjectAuthority` 创建工程并显式持有 generation；旧 `open_project_manager_for_paths` 同样清零。
- save 使用活动 generation，写入 default scene 后请求 Runtime `import_asset` 刷新；目标 root 与活动 generation 不一致时返回错误，不以重新打开工程掩盖状态漂移。当前 Runtime `import_asset` 仍执行 full import，transactional targeted replacement 已下沉 Runtime04 failure 并保持 open。
- welcome 投影只复制缓存。工程创建目标验证与 existing-project probe 由 Editor Job System 的 Background/Index job 执行；输入变化取消旧 ticket，只有当前 active generation 的结果能进入 `EditorStartupSessionDocument`。受控 Job 回归覆盖 A→B 重提后 A 迟到、退出取消、job failure 与 submit failure，不再只依赖源码字符串守卫。
- Runtime `AssetManager` 提供 manager-owned `current_project_source_path` 与 `current_project_asset_uris` 只读查询：full scan、artifact preload 与 watcher spawn 先在 cloned candidate 完成，随后在 project write lock 内整体发布活动 generation 的 ResourceManager/source-path index/ProjectManager/watcher owner；失败准备保留旧 generation。locator 解析按 `(scheme, path)` 查索引，`res://`/`package://` miss 均为 typed missing，preset 名称只复制 locator 集合后过滤，不为 1/100/1000 批量投影 deep clone 完整 `ProjectManager` 或逐 root stat。显式 preset save/load 仍使用 generation snapshot 并保留文件 I/O；`SceneProjectError` 通过 `EditorError::SceneProject` 保留 typed source chain。save 后的真正 targeted import 尚未关闭。
- UI asset 外部组件/样式提升、撤销与重做副作用都接收活动 `ProjectManager` 快照，删除最后三处交互路径 manager reopen。
- ProjectAuthority 结构守卫排除自身测试目录后继续检查生产源码的全部退休 UI/template/runtime-cache 词项，没有降低禁用规则。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|---|
| Editor10 M1/M2 | ProjectAuthority boundary + AssetRef 当前门 | `实现完成-静态门通过-受管行为门待执行` | 2026-07-17 | boundary guard 改为仅扫描生产源码；AssetRef 消费者继续以 GUID-first registry 记录并保留 locator fallback。当前 exact Cargo 结果尚未产生，不关闭 `failure-2026-07-12-project-asset-reference-full-gate-regressions.md`。 |
| Editor10 M1 | canonical project generation 与宿主复用 | `实现完成-静态门通过-独立复审中` | 2026-07-18 | `OpenedProject { project, summary }` 只解析一次 manifest/registry index，`open_prepared_project` 在同一实例上执行唯一 source scan；document `load_from_project/save_to_project`、watcher `start(&ProjectManager)`、UI asset promotion/editing 快照复用已落地。`open_project_manager_for_paths`、document `load_from_path/save_to_path` 与旧 `_from_path.rs` owner 全仓清零。 |
| Editor10 M1 / Performance01 | welcome/locator/preset 正常投影收束 | `部分实现-SourceIndex独立复审0/0/0-targeted import保持open` | 2026-07-18 | welcome snapshot 对 `ProjectAuthority`/`probe_draft`/`validate_for_creation` 搜索均为 0，并新增真实 cancellation/迟到 generation/job+submit failure 行为测试；layout preset list 段对 `ProjectManager::open`/`fs::read_dir`/`fs::read_to_string` 搜索均为 0。Runtime manager-owned source-path index 消除热路径完整 generation deep clone 与逐 root stat，新增 1,000 locator/100 preset、package remove、label、prepare failure 与 watcher lifecycle 回归，独立终审 `Critical/Important/Minor=0/0/0`；typed layout preset source-chain 回归已加入。transactional targeted import 的原子性、依赖边、duplicate GUID、compound topology 仍见 Runtime04 failure。 |
| Editor10 M1/M2 | 结构预算、scoped formatting 与差异卫生 | `通过` | 2026-07-18 | generation/preset 测试从已达 864 行的 `bootstrap_and_startup.rs` 拆至 folder-backed `project_generation_projection.rs`，行数分别为 719/255；相关 Rust `rustfmt --edition 2021` 与 scoped `git diff --check` 通过。 |

当前切片的独立终审已为 `Critical/Important/Minor=0/0/0`，但在 Windows 受管 Cargo exact gates 完成前仍只记为“实现完成/静态通过”；Runtime04 transactional targeted import 未关闭前 performance failure 同样保持 open。不将两个 Editor10 open failure 改名为 fixed，也不声称完整 Editor10 M1/M2 验收完成。
