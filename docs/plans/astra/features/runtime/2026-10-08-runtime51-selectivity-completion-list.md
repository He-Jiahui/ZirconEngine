# Runtime51 组合过滤完成列表

优化记录：[组合过滤使用最小索引集合](../../../optimize/zircon_runtime/51/2026-10-08-compound-query-selectivity.md)。

- [x] 当前生产查询选择最小类型、标签、包索引集合，保留交集谓词、去重不变量和规范排序。
- [x] 保留类型及前缀快路径、源文件与同源子资源移除语义。
- [x] 保留原 Windows 性能对照辅助函数体及原 22 个测试文件、65 个测试；新增四个组合查询回归。
- [x] 独立核心审查、限定文件格式和 diff 检查通过，代码已接入当前工作树。
- [ ] 合并异步批次的当前源代码原生编译、原测试和新增回归通过。
- [ ] 原 Runtime51 计划的全部规模、分配、CPU、RSS、I/O、延迟及产品验收达标。

源代码修复完成；测试与性能完成状态仍开放。已有本地验证结果不自动授予 Jenkins 或里程碑验收，未提交或推送代码。

上一 16 命令批次已确认在 Cargo 调用前因共享源码变化被拒绝；后继批次保留原用例和原性能门槛，重新绑定当前源代码。该拒绝不属于测试通过证据。

## 最新追加批次终态（2026-10-09）

12 命令批次 `runtime-editor-source-repair-batch-20261009-02` 已结束，首条 `three-package-check` 为真实 Cargo `101` 编译失败，绑定源码在该请求期间未变（`sourceUnchanged=true`），后续 11 条测试或性能命令均为 `not-invoked-prerequisite-failed`。原因已追踪到 Editor 测试迁移后的旧 ZUI include 和 ServiceHandle/Result fixture：24 个缺失 include 诊断与 4 个 E0282 级联构成 28 个 lib-test 错误，另 12 个 integration source diagnostics；JSON 42 条含两个 summary。实际证据 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/terminal-receipt.json` SHA256 `3ee0c9458fb7e1595a5c904da406981b9b6eaabcf780fc5771c8f5e52972a1ff` 与 `actual-compile-errors.json`。

这一结果没有执行本条的原生功能测试或性能 producer。此前 16 命令 source-drift 拒绝历史完整保留。本次后续控制器 fixture 已接入，UI 测试和生产合同最小修复继续审查；15 命令 successor 草案尚未绑定或启动。后续保持一批运行原测试与原性能矩阵，提交后继续功能修复候选，在实质源码/证据检查点一次核对终态。原测试、性能、产品、Jenkins 和里程碑完成项继续开放。
