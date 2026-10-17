# Runtime206：增量发布容量守卫范围修复

状态：`source_reviewed / native_performance_pending`。源码决策由 gpt-6.1-sol max 完成，独立静态复核由 gpt-6-luna max 完成。

[原容量优化计划](2026-09-15-incremental-publication-capacity.md)只要求增量发布 prepare 内四个有已知切片上界的集合预留容量。原 source-shape 用例在整个生产文件上禁止 HashMap::new() / HashSet::new()，把独立 compound 发布函数的合法分配也算作回归，形成 CR-ASSET-PIPELINE-SOURCE-GUARD-0001 误报。

[修复源码](../../../../../zircon_runtime/src/asset/pipeline/manager/assets/tests/incremental_capacity.rs)现在先确认 prepare / commit 签名各唯一、顺序正确，再以 brace depth 提取 prepare 自身的函数体；边界缺失、重复或未闭合均失败。四个预留容量正例、四个去重/物化正例，以及目标函数内无容量 map/set 的负例均保留。原普通用例新增反例：兄弟函数允许 new；目标任一 reserve 退化为 new、把正例移动到兄弟函数、目标内额外 new、无效函数边界均被拒绝。

生产文件没有修改，upper_bound、既有 ignored p95 producer 及模型辅助函数内容保持不变。本次修复测试合同；没有新增产品性能、CPU、RSS 或实测分配结论。

| 输入 | 字节数 | SHA256 |
| --- | ---: | --- |
| 最终测试源码 | 9053 | `070ddac59a7a1c2e1a290bf02220a5ca0463fa578059dcd02ddc29f066397aff` |
| 未修改的生产文件 | 20673 | `2dcc849548979429c4ae6b7b01474fab17c8c9fe5d6e617e025b6198e4ac2967` |

[补丁](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-incremental-capacity-guard.patch)、[核心证据](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-incremental-capacity-guard-evidence.json)、[静态 change-guard](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-incremental-capacity-guard-static.json)均已校验原始字节哈希。核心 fmt/diff 检查通过；root guard 实际子进程及外层均退出 0，errors/warnings/suppressed 均为 0；独立 delta 静态复核无阻断。21 条观察为静态源码/参考合同检查，不是 native 测试执行。

现有测试身份仍为 2 ordinary + 1 ignored。第14批在已有 Runtime 普通验证 arm 中补选以下两个用例；它们与旧所有适用 arm 的有效 ordinary 选择无重叠，普通匹配下限贡献 2，整组最终下限由其余修复的实际选择去重后确定：

- `asset::pipeline::manager::project_asset_manager::resource_publication::incremental_capacity_tests::optimization_batch_20260915_runtime786_incremental_publication_capacity_source_shape`
- `asset::pipeline::manager::project_asset_manager::resource_publication::incremental_capacity_tests::optimization_batch_20260915_runtime786_incremental_publication_capacity_upper_bound`

既有 `asset::pipeline::manager::project_asset_manager::resource_publication::incremental_capacity_tests::optimization_batch_20260915_runtime786_incremental_publication_capacity_p95` 为 ignored，不能计为 ordinary 通过数，也不能用这次静态复核声称其 P95 达标。所有原有性能门槛与 Runtime03 绝对预算保持开放，等待绑定实际请求、源码及环境的原生结果。

- [x] 修复 target/sibling 的守卫范围并保留退化反例。
- [x] 校验补丁、源码、未修改生产 witness 和静态证据。
- [x] 独立静态复核原身份与 producer 内容不变。
- [ ] 第14批实际原生用例通过、有效匹配数及全输入前后 seal 一致。
- [ ] 原始性能 producer 与所有适用性能门槛通过。
- [ ] 根据实际回执关闭历史 BUG 与整体优化验收。

批量异步交接前保持 native/performance pending；提交后继续独立功能修复，禁止等待或持续监控编译。未创建源码副本、编译产物、缓存、原生夹具或临时脚本；保留当前源码与必要证据，实际释放空间 0B。
