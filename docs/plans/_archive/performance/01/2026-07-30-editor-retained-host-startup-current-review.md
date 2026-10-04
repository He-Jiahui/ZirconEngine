---
related_code:
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/events/startup.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs
  - zircon_editor/src/ui/host/editor_asset_manager/change_stream.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/resource_manager_contract.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
tests:
  - inline tests: 0
  - rustfmt check: blocked by startup/state.rs external import ordering
  - scoped whitespace check: passed
  - current-source managed Windows Cargo pending
  - F0 cold/warm retained-host startup trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained-host startup当前源码复核（2026-07-30）

## 范围

`zircon_editor/src/ui/retained_host/app/host_lifecycle/startup.rs`与`startup/**`当前源 **20/20** 个Rust文件、**699** 行、**0** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`4d66dabcb6d2efe48782c22c47b08cf2bfab32ff08c1d70bbefa52989837f670`。9个tracked文件的外部未提交内容只读纳入，本轮未修改Rust。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| root/resources/session | 7/7 | 197 | 0 | manager解析、三流订阅与startup session串联 |
| state/construction | 5/5 | 231 | 0 | 固定规模host state与interaction状态装配 |
| template bridges | 3/3 | 104 | 0 | 一次加载9个builtin document，构造6个bridge |
| viewport/finalize | 5/5 | 167 | 0 | lazy viewport job绑定与首帧资产同步/事件丢弃 |

## 发现

- **PERF-MVP-499 / Editor09 P0重复全量首帧投影**：project activation已在`ui/host/project_access.rs`调用一次`editor_asset_manager.refresh_from_runtime_project()`；host state构造完成后，`finalize_startup_host`又无条件调用`sync_asset_workspace()`，其第一步再次执行同一refresh。当前`sync_from_project`每次都遍历完整registry、逐asset读meta、为ready asset读artifact并提取references，再重建UUID/locator maps、catalog与preview scheduler。因此同一project在首帧前至少执行两次Editor09全量catalog输入读取和重建，成本、I/O与峰值RSS均随资产数增长。
- 第二次refresh之后，`sync_asset_workspace`还同步发布catalog snapshot、调用`ResourceManager::list_resources()`、刷新selected details并构造完整chrome以收集visible UUID和提交preview。`list_resources()`每次深clone全部`ResourceRecord`并以`primary_locator.to_string()`生成排序key，启动稳定面仍支付O(N log N)排序、String分配与宽record复制；继续归 **PERF-MVP-500**，不得在retained host建立第二套资源索引。
- **Editor09 bootstrap事件O(A)清空**：`resolve_startup_managers`在project open之前订阅asset/editor/resource三条change stream，open/import和两次catalog投影产生的bootstrap事件随后才被丢弃。asset/resource receiver先读取`len()`再逐条`try_recv`；EditorAsset mailbox虽一次加锁执行`VecDeque::clear + HashMap::clear`，仍需逐项析构最多512个pending key。它避免了第二轮presentation replay，却在首帧前增加与bootstrap事件数线性相关的主线程工作。
- 修复不能简单删除第二次refresh或无条件后移订阅：当前没有跨Runtime04/Editor09的`project/catalog generation`相等证明，删除refresh可能发布旧catalog；订阅后移又可能丢失project commit后并发事件。目标是project activation返回同一immutable catalog generation/token，retained host按generation只做一次publish，并让订阅从已提交watermark开始；启动丢弃应成为O(1) cursor/watermark推进，而不是逐记录清空。
- 正向边界：state assembly只构造固定数量的空集合、bridge和交互状态；六个template bridge共享一次builtin runtime/document load，没有重复读取同一template。viewport仅绑定`EditorJobSystem`，`RenderFramework`解析保持lazy job，没有把GPU/device初始化重新放回startup caller。
- 当前20文件没有行为或性能测试。即使后续Cargo通过，也必须以1/1K/100K资产和bootstrap storm产品计数证明refresh次数、registry/meta/artifact visits、event discard wall与首帧p95，才能进入`review.md`。

## 参考与目标

- Bevy `dev/bevy/crates/bevy_asset/src/server/mod.rs:324-325,573-603`复用已加载path的handle而不再创建load task，并在潜在阻塞task前释放asset-info锁，再交给`IoTaskPool`。Zircon应复用同一generation/ticket和Runtime11预算，不能把第二次全量refresh改名后搬到私有线程。
- Godot `dev/godot/editor/file_system/editor_file_system.cpp:1083-1140,1702-1721`显式合并正在进行的scan请求，并把允许异步的扫描放到低优先级线程；其首扫仍受主线程API限制，说明Zircon验收必须逐阶段记录caller/worker wall，而不能只看总启动时间。

Runtime04负责在锁外构造一次project/catalog-input generation并短提交；Editor09从该generation发布一次immutable catalog/resources projection；retained host只消费generation token与MVP visible/selected rows。Runtime11提供有界、可取消、single-flight的I/O/import/reference/preview jobs。三类change stream必须从commit watermark开始，保留rename/remove/failure顺序与last-good，不得靠静默清空掩盖未消费事件。

## 动态验收

按assets `1/1K/100K`、ready artifacts `0/50/100%`、bootstrap events `0/512/100K`、visible rows `0/50/1K`运行cold/warm/unchanged/1% change、并发watch与失败回滚；记录`refresh_from_runtime_project`调用数、registry/meta/artifact/reference visits、catalog/resource builds、`list_resources` clone/sort/key bytes、bootstrap pending/drained/queue age/discard wall、UI/worker wall、锁wait+hold、F0首帧p50/p95与RSS。

验收要求：project/catalog generation每次activation最多构造/发布1次，warm unchanged全量refresh/build/read=0；startup event handoff为O(1) watermark推进且commit后的并发事件不丢；resource projection近visible/page delta，stable全量clone/sort=0；长I/O不在UI caller或generation/editor锁内。managed Cargo、规模counter、current-source independent review与F0 trace完成前保留在`pending.md`，不进入`review.md`。
