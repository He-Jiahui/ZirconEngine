# Editor17 M3.3 Settings persistence health foundation

## 目标与状态

- 状态：`source_complete_static_green / change_driven_health_authority_complete / failure_notification_projection_complete / operator_retry_ui_complete / file_generation_identity_complete / managed_validation_pending`。
- 目标：让 User/Project settings 文档的准入、排队、耐久和失败状态由 Context-owned mutation coordinator 主动发布，而不是由 retained UI 每帧读取 Runtime11 diagnostics 或保存 persistence ticket。
- 非声明：本切片没有完成 immutable file projection/digest、锁外整文档编码、Cargo、真实 worker failure 产品输入与窗口点击、WPR、性能或功耗验收；强类型 file-generation 身份不等于对应字节已经被封存。

## Current-source 与参考重审

- `SettingsMutationCoordinator` 已是 User store、active Project binding、pending ticket 和 typed retry 的唯一 owner，但 worker 终态此前只在显式 `retry_pending` 时读取；UI 若直接补 `ticket.terminal()` 或 `diagnostics()` 会复制生命周期并退化为轮询。
- Runtime11 `BoundedKeyedIoAdmission::observe_terminal` 已在 admission 激活前注册 observer，成功、失败、取消、supersede 与 shutdown 都由 lane 锁外通知；Editor17 无需增加私有 worker、channel 或 timer。
- Unreal `ISettingsSection` 把 `GetStatus`/`OnStatus`、`CanSave` 与 `Save` 保留在 section owner；Zircon 对齐“设置 owner 发布状态、视图只消费”的边界，但仍由 Runtime11 执行 I/O，并以 immutable typed snapshot 代替 UObject/widget 持有保存状态。

## 实现决策

- 新增 `mutation/health.rs`，由 coordinator 内的 `SettingsPersistenceHealthAuthority` 保存 User/Project 两个固定文档槽、health generation、active observation token、`SettingsFileGeneration` 和 `Unavailable/Ready/Queued/Durable/PendingAdmission/Terminal` 状态。
- 每次 submit/retry 先发布新的 observation token，再在 Runtime11 admission 上注册 terminal observer。回调必须同时匹配物理文档 identity 和 token；Project bind/clear 会更换或清除 identity，因此旧 project、被 coalesce 的旧 ticket 或迟到 retry callback 不能覆盖当前状态。
- health subscriber 在释放 state/subscriber mutex 后收到 immutable `SettingsPersistenceHealthSnapshot`。subscriber 不拥有 ticket、不读取 filesystem/lane diagnostics，也不得同步回写 settings。
- Context composition root 继续只有一个 `EditorNotificationService`，现以 `Arc` 共享给 job progress observer、Context accessor 和 settings health subscriber。只有 `PendingAdmission` 与 worker `Failed` 转为带 health generation 的独立 error toast；Queued/Durable/Superseded/Cancelled 不产生噪声。英中资源使用同一三项 localization key。
- 既有 `retry_pending(scope)` 仍是唯一 retry 命令边界。Settings 标题栏仅在 Project/User 的 `PendingAdmission` 或 worker `Failed` 状态显示短状态与 `refresh-outline` 图标，优先 Project、其次 User；点击 action 只把 scope 交给 host runtime，再调用 coordinator typed retry，不持有或重建 ticket。
- health UI 投影只读取固定的 Project/User 两槽并按 health generation 写入 retained bridge。普通 mutation 和手动 retry 在各自既有刷新中合并 health 属性；worker failure 只在新 error toast 使 notification snapshot 变化时捕获 health，并与通知共用一次 retained-tree refresh。稳定通知帧不读取 health，不调用 lane diagnostics。
- persistence ticket 现以 `(scope, physical-target hash, SettingsFileGeneration)` 为 lane/receipt 身份；进程单调分配器避免相同 project path 重新绑定时 generation 倒退。authority revision 只保留为诊断字段，health、retry receipt 与 Runtime11 generation 都使用 file generation。Request 自带原始 `SettingsStore`，`retry(ticket)` 不再接受外部 Store；deferred admission 同样保留原 Store 和 file generation，不能用旧 lane key 写到新路径。

## 复杂度与验证边界

- 每次 health transition 只更新 User/Project 中一个固定槽并克隆 Copy-sized snapshot，时间和保留内存均为 O(1)；terminal callback 不读取 authority、registry、filesystem 或 Runtime diagnostics。
- TDD RED：5 项静态合同首次为 3 failures + 1 missing-owner error，随后产品 subscriber 合同再以 1 missing-owner error 失败。
- retry UI TDD RED：新增 5 项产品合同首次为 3 failures + 2 missing-owner errors；GREEN 为 retry 5/5。file-generation identity TDD 首次 4/4 failures，随后 GREEN 4/4；累计 health 5/5、既有 Settings 窗口合同 15/15、Editor17 全发现集 48/48；英中 i18n 与 Workbench ZUI TOML 解析、限定 rustfmt 与 scoped diff 通过。新增 Rust 行为回归覆盖 worker terminal 主动发布 Durable、deferred admission 保留同一 file generation、target-bound retry，以及 pane 转换保留 health generation/scope/status，但受管 Cargo 未执行。
- owner 预算：`mutation.rs` 750 行、`mutation/health.rs` 302、`persistence.rs` 494、`io.rs` 496、Context subscriber 91、UI health projection 74、Settings bridge 655、Settings painter 690、health painter 58，均低于生产文件 800 行 review 警告。
- 未生成耗时、RSS、WPR 或功耗数据；本切片是状态架构修复，不是性能优化结论。

## 后续硬前置

1. 在已完成的 file-generation/target identity 上封存 immutable file projection 与 encoded digest，并把完整编码移出 authority/project lock；worker 成功必须回报精确 durable generation，失败保留 dirty generation。实现前的 current-source/Unreal 重审已写入性能计划，动态 profile 仍待受管产品入口。
2. 执行受管 Cargo、deferred/failed/retry/project-switch/shutdown 产品链、真实窗口命中/绘制和 F0/F4 trace，确认通知、status 与 shutdown 决策不丢终态。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-26 | M3.3 change-driven settings persistence health foundation | `source_complete_static_green / change_driven_health_authority_complete / failure_notification_projection_complete / operator_retry_ui_complete / file_generation_identity_complete / managed_validation_pending` | 复用 Runtime11 admission terminal observer，由 Context coordinator 固定 User/Project health 槽并以 document identity + observation token 拒绝迟到终态；锁外 subscriber 把 PendingAdmission/Failed 投影为英中 error toast。Settings 标题栏现显示 Project-first/User-second 的失败状态和可直接命中的 retry 图标，action 只调用 typed `retry_pending(scope)`。Persistence request/ticket/health/retry 已硬切为 target-bound `SettingsFileGeneration`，authority revision 仅为诊断；Request/Deferred 自带原 Store，重试不能改写物理目标。file-generation 4/4、health 5/5、retry 5/5、Settings 窗口 16/16、Editor17 48/48、限定 rustfmt 与 owner budget 通过；Rust 行为未由受管 Cargo 执行，immutable file projection/digest、锁外编码、真实产品 failure/retry、性能/功耗仍待完成。 |
