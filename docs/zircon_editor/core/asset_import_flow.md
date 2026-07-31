# Editor Asset Import Flow

`zircon_editor::core::asset` façade 的 import flow API 是 Runtime 资产导入能力的编辑器编排层。它不拥有
importer、worker pool、registry、`.zmeta`、source digest 或 artifact 写入；这些事实继续由
`zircon_runtime::asset::AssetManager` 及其内部 pipeline 负责。

## 边界

- `EditorAssetImportFlow` 接收 Runtime `AssetManager`、Plan14 `EditorJobSystem` 与共享
  `EditorAssetIndex` 投影。
- 每个新 `(UUID, normalized AssetUri, Runtime source digest)` generation 以
  `JobCategory::Import` 提交一次；等价 watch/digest/manual 请求共享同一个 job id、完成结果和原因集合。
- 同一 UUID 的不同 generation 在活跃期复用一个 `MutexGroup`，因此路径迁移前后的来源不会并发写；
  generation key 只消费 Runtime 已规范化的 URI 与 digest，不创建编辑器侧内容摘要权威。
- `EditorAssetImportReason` 只记录触发原因：watch、既有 digest 失配或手动重导入。它不计算
  digest，也不覆盖 `.zmeta` 的 import settings。
- job 直接调用 `AssetManager::import_asset`；Runtime 返回的 `AssetStatusRecord` 原样进入
  `EditorAssetImportResult`，编辑器不重建平行状态记录。

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

## 使用约束

- 调用方持有可克隆的 `EditorAssetImportTicket`，通过 `try_result` 或 `wait` 观察共享结果；取消统一调用共享
  `EditorJobSystem::cancel(ticket.id())`。
- UI 进度只消费 Plan14 job progress/event 投影；不得轮询磁盘或另建导入线程。
- Runtime registry/watch 更新完成后由 M2.1 `EditorAssetIndex` 重新投影结果；import flow 不直接写
  registry 行、sidecar 或 artifact。
- folder-backed owner 已私有挂载到 `core/asset/mod.rs`；调用方只使用 `core::asset` 的 typed 重导出，
  不暴露第二条 `core::asset::import_flow` 公共兼容路径。挂载后的受管 Cargo 与 milestone commit 尚未完成。

## 验证

- 静态合同：`python -m unittest tools.tests.test_editor09_import_flow_contract -v`。
- Rust 合同覆盖成功、typed Runtime failure、未知 URI、10,000 次等价 generation storm 单 job、
  admission-pending 快速失败 observer、registry revision 重验、UUID phase token、失败重试、
  entry/byte/oldest-age 三预算、hot-key TTL、超大成功结果不缓存、共享取消、panic/shutdown 清理及路径迁移。
- 必须在 Coordinator01 冻结且闭合 Cargo local-path manifest graph 的 source copy 上运行聚焦 Rust
  门；共享工作树上的偶然 GREEN 不计验收。
