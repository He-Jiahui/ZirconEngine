# Runtime02：事件容量计数重复查表优化记录

日期：2026-10-09。状态：单函数源码优化已采用；原生功能及性能待验证。

## 已采用的优化

AdmissionLedger::charge 直接取得 subscribers 与 topics 两个不同字段中的 mutable entry。成功路径由四次 map 查询减少为两次，去除临时 topic Arc 的 clone/drop；不插入、裁剪或改变锁边界。

subscriber、topic、global 的三个 checked-add 全部完成后，按原 subscriber→topic→global→peak 顺序提交。未知订阅者、缺少 topic、任意 overflow、零值及关闭/poison 行为保持；失败不修改 retained 或 peak 计数。函数外的字节、发布预检与提交/释放顺序保持。

## 源码验证证据

[独立核心源审](../../../../../.codex/outbox/runtime02-admission-charge-entry-reuse-core-review-v1/review.json)和[实际采用回执](../../../../../.codex/outbox/runtime02-admission-charge-entry-reuse-v1/actual-canonical-apply.json)。

实际应用一文件；45 个输入两次核对、44 个非修改输入保持，Rust 2021 格式与 scoped diff 检查通过。共享索引、旧索引锁内容及元数据、Cargo 和控制配置保持。最初 Root 校验脚本误数三个 checked-add 的安全拒绝记录保留，当时没有源码写入。

实际采用回执 SHA-256：642b5deab7a1875039c9791fd9175629f9d04cd6308d1c4c753672c75b734195。

## 待验证

原 17 个业务测试和 7 个 EventBus Release producer 的名称、函数、断言及预算均保持，现有 15 项批次 #2 / #9 已覆盖，不增加 arm、selector 或用例。

减少查表和 Arc 计数是源码事实，尚无 allocation、CPU、锁等待、延迟或 RSS 实测。原 tasks 1/2/8/32、DLL 1000、fanout 0/1/2/5/100、payload 64/4K/256K、batch 1/32/1K，以及原 10K/100K/1M 矩阵及完整预算仍待通过；没有 native、Jenkins 或性能达标声明。

对应[计划完成列表](../../../astra/features/runtime/2026-10-09-runtime02-charge-completion-list.md)。
