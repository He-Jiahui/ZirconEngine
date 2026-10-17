# Runtime160：监听映射不确定性进入原批次重扫

记录时间：2026-10-15T06:55:36.289247+00:00。核心实现：camera_mesh_core / gpt-6.1-sol max；独立复核：batch12_evidence / gpt-6-luna max。

原映射将不完整 RenameMode::Both、单端 rename 或映射失败折成空增量，loop 无法知道需要重扫。现在私有 MappedWatchEvents 同时返回 events 与 requires_reconciliation；不增加 public batch 字段或第二个重扫 owner。

两端可映射的 Both 仍产生一个 Renamed。相关路径的缺端、多端、单端/未知 rename 或映射失败会请求 reconciliation，不伪造 Modify。全部在 root 外、meta sidecar 或已确认事务临时路径继续忽略；无路径事件因范围未知而请求重扫。普通 directory modify echo、compound root 和有效文件事件维持原策略。

loop 在 fold 之前合并 flag，以 ingress received_at 开始原批次；空 changes 但 flag=true 也沿原 quiet/max-latency/stop/disconnect flush 发出 callback，然后复位。有效事件 FIFO、mapped raw_event_count、容量/字节 rollback、overflow 计数和 on_error 语义保留。

下游 watch_dispatch/runtime 原有分支已会在 reconciliation 时清除部分增量并全量同步，此次未修改它们。新增测试覆盖真实 ingress -> loop -> callback，不宣称已覆盖完整下游运行时 E2E。

5 个既有文件已冻结；mapper leaf7 ordinary+1ignored，watcher leaf12 ordinary，合计19 ordinary = preserved14 + new5。新例覆盖 rename/known-ignore/map-failure、flag批次复位、stop 和 ingress disconnect。原14 body/prefix保留，已有调用点只做 outcome.events 适配。

原 B16 Runtime ordinary filters 未覆盖这19项。与 AnimationRuntime5个真新增及反射读取16个既有 ordinary 合批时，Runtime floor121+5+19+16=161，required64+5+19+16=104；在同一 Runtime invocation 追加19个实际 watch FQN 与16个既有反射读取 FQN，原所有 argv/required/per-prefix（包括 null）前缀不删改。

root scoped change guard 实际 outer0、child0、0errors/0warnings；独立 source+packet peer 已完成。13 个只读调用链/模块/producer witness 和1个原计划 pin 与当前字节一致；canonical Git before 与实际 admission raw 也匹配，scoped patch reverse --check child0。

原 RUNTIME369_WATCH_EVENT_MAPPING_CAPACITY_BENCH_V1 仍为17paired/4096paths/64rounds、optimized P95<=legacy70%；它只测 capacity。原 RUNTIME88_WATCH_FOLD_STACK_SCRATCH_BENCH_V1 为17paired/16384events、P95<=legacy95%，只测 scratch；两者都不能代表整个 reconciliation 修复或 watcher storm 性能。

原生回归、两项及全部原性能/resource/Runtime03 AND 门槛待同源 Jenkins 回执。跨平台 path protocol、alias/encoding、rename cookie/chain/replay、全 watcher burst/overflow/storm 及完整 FS-G11/P1-039 qualification 保持 OPEN；词法 root 与已有 lossy path 协议未在本次改写。

[原计划](160-runtime-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-current-source-review.md)；[补丁](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime160-watch-mapping-reconciliation.patch)；[源码证据与完整 FQN](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime160-watch-mapping-reconciliation-evidence.json)；[静态检查](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime160-watch-mapping-reconciliation-static.json)；[Astra 完成列表](../../astra/features/runtime/2026-10-14-runtime160-watch-mapping-reconciliation-completion-list.md)；[冻结 Runtime03 绝对 P95 标准](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-absolute-p95-criteria.json)。

产物：仅预登记补丁、元数据、静态报告和计划记录；未创建编译/cache/临时 helper/native fixture/源码副本。实际释放0B，必要验收和失败证据保留。历史 UNKNOWN 进程/回执不因本记录而关闭。
