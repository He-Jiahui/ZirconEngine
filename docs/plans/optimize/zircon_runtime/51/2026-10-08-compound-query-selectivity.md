# Runtime51：组合过滤使用最小索引集合

对应 [原优化计划](../51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md)。当前状态：生产代码已接入，原生测试与性能验收待结果。

## 已完成的代码修复

- 类型与标签、包组合过滤时，将类型、标签、包的倒排集合一起比较，借用最小集合生成候选；保留全部交集谓词和规范路径顺序。
- 类型单独过滤及类型与路径前缀过滤继续使用已有类型快路径。候选在输出收集之前过滤，减少宽候选集合的临时引用存储。
- 保留前缀查询已接入的借用路径区间遍历，继续维护同源子资源、移除源文件后的查询语义。
- 原 Windows 性能用例使用的 50 行 `candidate_entries` 函数体逐字保留，仅在 Windows 测试中编译；生产查询使用新的 `matching_candidate_entries`。原 22 个测试文件和 65 个测试保留。
- 新增四个回归用例，覆盖宽类型与稀疏标签/包、规范排序、非空集合的空交集、缺失集合、类型前缀与离线移除。

独立核心审查修正了初版补丁会间接改变旧性能基线的问题。最终补丁 SHA256：`d468604fa19cf37399140051781e2be74f3bca481890e2cc9ea24ebb5f703239`。接入后格式和限定文件的 diff 检查通过，Git 索引与控制文件保持原状。

## 验证与剩余验收

上一批将组合过滤回归与 Runtime 任务/Loader、Editor Activity、App 边界及已有 Release 性能用例纳入同一个 16 命令批次。该批终态回执确认三处共享 Editor 源码变化，首条命令在调用 Cargo 前被拒绝，其余命令未调用；原生测试和性能尚无通过证据。后继批次在接入 EventBus、Runtime profiling 和 Editor 事务脏状态修复后，合并同类库测试与原 Release 证据生产者。Jenkins 当前缺少有效启用记录，使用已有独立本地入口；本地结果不代表 Jenkins 或里程碑验收。

- [x] 当前源代码接入、独立核心审查、格式与 diff 检查。
- [ ] 当前源代码原生编译及原测试、新增回归实际通过。
- [ ] 原计划的 10K/100K/1M 查询与重建、依赖/引用及高扇出工作负载达标。
- [ ] 原计划要求的 CPU、分配、RSS、I/O 和 p50/p95/p99 完整证据及产品集成验收。

源代码结构说明扫描量减少的原因；尚无实际测量支持延迟、内存或分配达标。原数值预算和历史失败继续保留。

证据位于 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/runtime51-core-selectivity-review-v1/`：`review.json`、`binding-strict-baseline.json`、`actual-canonical-apply.json`。原辅助函数体前后 SHA256 均为 `0f96dbae368729a52dd7c8c14d4d8efb1bb7542e03c180e4f02634ada360e032`。

批次目录：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261004/batch-inputs-v1/`。原请求终态回执 `terminal-receipt.json` 已在后续代码交接点读取一次并保留；该批未产生 Cargo 测试日志。后继异步批次目录为 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/`，当前清单已准备，启动身份和原生结果须以实际回执为准。编译期间继续后续修复。

## 最新追加批次终态（2026-10-09）

12 命令批次 `runtime-editor-source-repair-batch-20261009-02` 已结束，首条 `three-package-check` 为真实 Cargo `101` 编译失败，绑定源码在该请求期间未变（`sourceUnchanged=true`），后续 11 条测试或性能命令均为 `not-invoked-prerequisite-failed`。原因已追踪到 Editor 测试迁移后的旧 ZUI include 和 ServiceHandle/Result fixture：24 个缺失 include 诊断与 4 个 E0282 级联构成 28 个 lib-test 错误，另 12 个 integration source diagnostics；JSON 42 条含两个 summary。实际证据 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/terminal-receipt.json` SHA256 `3ee0c9458fb7e1595a5c904da406981b9b6eaabcf780fc5771c8f5e52972a1ff` 与 `actual-compile-errors.json`。

这一结果没有执行本条的原生功能测试或性能 producer。此前 16 命令 source-drift 拒绝历史完整保留。本次后续控制器 fixture 已接入，UI 测试和生产合同最小修复继续审查；15 命令 successor 草案尚未绑定或启动。后续保持一批运行原测试与原性能矩阵，提交后继续功能修复候选，在实质源码/证据检查点一次核对终态。原测试、性能、产品、Jenkins 和里程碑完成项继续开放。
