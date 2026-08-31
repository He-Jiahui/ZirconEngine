# Editor Asset Import Flow

`zircon_editor::core::asset` façade 的 import flow API 是 Runtime 资产导入能力的编辑器编排层。它不拥有
importer、worker pool、registry、`.zmeta`、source digest 或 artifact 写入；这些事实继续由
`zircon_runtime::asset::AssetManager` 及其内部 pipeline 负责。

## 边界

- `EditorAssetImportFlow` 接收 Runtime `AssetManager`、Plan14 `EditorJobSystem`、共享
  `EditorAssetIndex` 投影与 `EditorContext` 持有的同一个 `EditorLogService`；不得创建资产导入专用日志存储。
- 每个新 `(UUID, normalized AssetUri, Runtime source digest)` generation 以
  `JobCategory::Import` 提交一次；等价 watch/digest/manual 请求共享同一个 job id、完成结果和原因集合。
- 同一 UUID 的不同 generation 在活跃期复用一个 `MutexGroup`，因此路径迁移前后的来源不会并发写；
  generation key 只消费 Runtime 已规范化的 URI 与 digest，不创建编辑器侧内容摘要权威。
- `EditorAssetImportReason` 只记录触发原因：watch、既有 digest 失配或手动重导入。它不计算
  digest，也不覆盖 `.zmeta` 的 import settings。
- job 直接调用 `AssetManager::import_asset`；Runtime 返回的 `AssetStatusRecord` 原样进入
  `EditorAssetImportResult`，编辑器不重建平行状态记录。
- 外部模型源走同一 flow 的 `submit_model_source`，只提交 Runtime 拥有的 compound import transaction，
  并返回拥有唯一 Plan14 result receiver 的 `EditorModelImportTicket`；Runtime receipt 返回前，编辑器不自行
  暂存源文件、派生子资产或写 registry。

## 生命周期与失败

提交先在 `EditorAssetIndex` 中捕获带 registry revision 的 URI→UUID/source generation，释放 index 锁后才在
flow state 中准入或附着共享 flight；返回 ticket 或创建 job 前会在 index 锁内复验 generation，首个 UUID
generation 的复验与 `Importing` 标记在同一临界区完成。flow state 以 UUID
`Starting → Ready → Clearing` token 阻断 begin/clear 交界，且从不与 index 同时持锁；backend/job submit
也不在任一状态锁内。每个真实 job 持有 RAII
`ImportLease`：成功、Runtime typed failure、运行期取消、panic，
以及尚未运行就被取消或因 job system shutdown 被拒绝时，lease 都会释放。只有该 UUID 的最后一个
活跃 generation 释放时才清除 `Importing`，因此 registry 路径迁移中的同 UUID 请求
不会被较早完成项提前清状态。

lease 先终结 flow state、必要的 index clear 和 UUID token，再发布共享结果；因此 `wait`/`try_result` 不会在
生命周期清理前返回。成功 generation 在有界 retained-flight 预算内复用；失败、取消和 panic generation
立即逐出，允许后续请求重试。`EditorAssetImportAdmissionLimits` 同时限制 flight 数、估算驻留字节和最老
活跃 flight 年龄；成功结果中的动态 `AssetStatusRecord` 字符串计入 byte budget，超预算结果不缓存。
completed TTL 在后续访问时惰性逐出，即使访问的是同一 hot key 也不能绕过。达到预算时返回 typed submit
error，不能把压力转移到 Plan14 队列。

Runtime `CoreError` 通过 `JobError::Failed` 保留 typed source；提交前的未知 URI、索引变化与 Plan14
拒绝分别由 `EditorAssetImportSubmitError` 表达。不能用字符串错误或 UI toast 作为状态真源。

`EditorAssetImportDiagnostics` 向共享 `EditorLogService` 投影一次终态：普通资产由真实 job 的
`ImportLease` 在发布共享 flight 结果前记录；模型导入由 job 与提交方共享的 terminal diagnostic guard
记录。guard 在 submit 成功后 arm，job 正常完成或 pending cancellation 导致的 Drop 都只能提交一次结果；
若 job 在 submit 返回前完成，结果先暂存并在 arm 时投影，提交拒绝则覆盖未 arm 的取消候选并记录 rejection。
因此日志正确性不依赖 UI 是否轮询或持有 ticket。成功为 `Info`，Runtime 返回未导入状态或取消为
`Warning`，失败、提交拒绝与 panic 为 `Error`，来源固定为 `LogSource::import()`。已知 `AssetUri` 始终附带
typed asset jump。共享 flight 或 terminal guard 分别充当完成身份，重复观察和重复终态投影不会重复写日志；
日志容量、字节预算与事件回压继续由单一日志服务负责。retained host 只消费模型 receipt 以刷新 catalog、
解析资源并实例化场景对象，不构造日志条目。

关闭工程时，retained host 先请求取消未完成的模型导入，再异步等待该 ticket 到达 terminal。等待期间不释放
Runtime project session；poll 消费终态后自动续接关闭。若 Runtime compound transaction 已越过提交边界，
其成功 receipt 仍按 durable 结果处理，但关闭路径跳过 catalog 刷新和场景实例化，然后才释放工程。

## 使用约束

- 调用方持有可克隆的 `EditorAssetImportTicket`，通过 `try_result` 或带显式 deadline 的 `wait_until` 观察共享
  结果；取消统一调用共享 `EditorJobSystem::cancel(ticket.id())`。
- 模型调用方持有不可克隆的 `EditorModelImportTicket`，通过 `try_take` 或带显式 deadline 的 `wait_until`
  消费唯一结果；不得提取或复制其底层 `JobTicket<ProjectImportReceipt>`，也不得承担终态日志投影。
- UI 进度只消费 Plan14 job progress/event 投影；不得轮询磁盘或另建导入线程。
- Runtime registry/watch 更新完成后由 M2.1 `EditorAssetIndex` 重新投影结果；import flow 不直接写
  registry 行、sidecar 或 artifact。
- folder-backed owner 已私有挂载到 `core/asset/mod.rs`；调用方只使用 `core::asset` 的 typed 重导出，
  不暴露第二条 `core::asset::import_flow` 公共兼容路径。挂载后的受管 Cargo 与 milestone commit 尚未完成。

## 验证

- 静态合同：`python -m unittest tools.tests.test_editor09_import_flow_contract -v`。
- Rust 合同覆盖成功、typed Runtime failure、未知 URI、10,000 次等价 generation storm 单 job、
  admission-pending 快速失败 observer、registry revision 重验、UUID phase token、失败重试、
  entry/byte/oldest-age 三预算、hot-key TTL、超大成功结果不缓存、共享取消、panic/shutdown 清理、路径迁移，
  以及日志成功/警告/失败路由、重复观察去重、有界 completion storm、模型 pending cancellation 无观察
  单次诊断、submit/terminal 竞态与工程关闭等待模型终态。
- 必须在 Coordinator01 冻结且闭合 Cargo local-path manifest graph 的 source copy 上运行聚焦 Rust
  门；共享工作树上的偶然 GREEN 不计验收。
