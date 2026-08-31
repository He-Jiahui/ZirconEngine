# Editor03 Scene Document History Routing Hard Cut

## 产出记录与时间

- 2026-08-24 21:32 +08:00 | 场景文档历史路由 hard cut | `实现完成，未验收`：调研当前 `DocumentLifecycleAuthority`、`SceneDocumentRoute`、`EditorState`、启动与保存链路，并参考 Unreal CurveEditor `FTransactionManager` 的“事务只关联已拥有对象”原则。生命周期是唯一场景身份 authority；启动、picker 新建/打开及 AlreadyActive 路由提交后，状态层只保存已提交 `DocumentId`。场景命令、gizmo release、undo/redo、snapshot、save `saved_top`、close dirty prompt 与 authoring trace 全部改为 `HistoryContextId::Document(document)`，workbench 范围 `Global` 命中为 0；未绑定场景拒绝编辑，replace/clear 只 finalize 当前文档历史。新增状态路由回归、未绑定拒绝回归与场景提交新建/AlreadyActive source contract。`rustfmt --check` 和 scoped `git diff --check` 通过；受管 D 盘 `zircon_editor` build/test 均被外部 `zr_rhi/src/surface.rs:233-234` E0499 提前阻断，未进入 editor 编译或测试，故不提交、不发送企微，durable journal append/recovery 也仍是 Editor17 独立未完成工作。

- 2026-08-24 21:55 +08:00 | scene lifecycle journal binding | `实现完成，未验收`：在已完成的 `DocumentId` history hard cut 基础上，项目会话新增唯一 journal coordinator；它从自身 project root 和物理 scene source 路径建立 durable identity，启动与 picker 路由均在 authoring world/lifecycle publication 之前完成绑定。场景创建发布、catalog、安装失败均释放 reservation binding；切换后的 Closed document 释放会话 binding。新增工程外路径拒绝、激活后 binding 存在、binding failure 不激活场景的回归。未接通 transaction append/recovery；局部 Rustfmt、scoped diff 检查通过，Cargo 仍受外部 `zr_rhi` E0499 阻断，不能提交或发送企微。

- 2026-08-24 22:07 +08:00 | 受管 Cargo 复验 | `外部阻塞，未验收`：受管 D 盘 target `f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d` 的 `cargo build -p zircon_editor --locked` 退出码为 `1`。日志已进入 `Compiling zircon_editor`，但依赖 `zr_rhi/src/surface.rs:233-234` 的两个 E0499 先终止 build；未取得 editor crate 或本轮 regression 的 Cargo 成功结果。`rustfmt --check`、scoped `git diff --check` 和生产 `activate_scene` 调用清零静态扫描通过，仍不具备提交/企微条件。

- 2026-08-24 22:18 +08:00 | project journal authority ownership | `实现完成，未验收`：重审启动与恢复会话后，修复 `publish_document_startup_session` 与 `complete_project_open` 重复初始化 journal coordinator 会覆盖现有 session binding/writer 的风险。协调器现在公开不可变 project root；manager 对同根请求复用当前 authority，对异根请求返回 typed `DocumentJournalCoordinatorError::ProjectRootConflict`，不允许静默替换会话 owner。两个生产调用点均传播该失败，激活回滚路径继续清理 coordinator。尚未连接 transaction append，且 Cargo 仍由外部 `zr_rhi` E0499 阻断，故不提交、不发送企微。

- 2026-08-24 22:25 +08:00 | D 盘定向构建与回归复验 | `外部阻塞，未验收`：受管 `validate-matrix.ps1 -Package zircon_editor -LibTests -TestFilter 'document_journal_coordinator|scene_document'` 完整执行；`Cargo build` 和 `Cargo test` 均为 exit `101`，均在 `zr_rhi/src/surface.rs:233-234` 的 E0499 停止，后者未进入任何 `zircon_editor` 测试。确认该外部阻塞仍有效；本切片只有 Rustfmt、scoped diff 与静态 routing 检查可作为本地验证，不提交、不发送企微。
