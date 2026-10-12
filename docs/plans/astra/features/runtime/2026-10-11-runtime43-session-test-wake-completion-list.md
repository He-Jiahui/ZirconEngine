# Runtime43：Session 测试 wake 编译修复完成列表

日期：2026-10-11。[优化记录](../../../optimize/zircon_runtime/43/2026-10-11-session-test-wake-construction-compile-repair.md)。状态：源码修复已实际采用，原生与性能验收未完成。

- [x] 独立核对实际 `#[cfg(test)]` 模块路径、registry 重导出和等价 disabled wake / SessionSlot / handle 分配 / 失败清理调用链。
- [x] 17 项当前守卫与冻结 binding 完全一致后，一次采用单测试文件的四个 hunk；两处 import 与构造改用已有 `registry::insert_session`。
- [x] 保留原五个测试、断言、lifecycle、FFI 与 teardown 正文；生产可见性与 helper 无变化，新增测试和 arm 为零。
- [x] 实际 canonical Rustfmt 2021、scoped diff 检查完成；shared index 尚未跟踪该文件，补充的 canonical no-index whitespace 检查无诊断，冻结 patch reverse-check 为 `0`。
- [x] 采用后其余 16 项源码、controls、shared index 与旧空 Git lock 的内容和元数据保持原样；未生成源码副本。
- [x] 静态核对 V9 第 2 项原 26 个 OR 筛选与两条受影响原用例交集为零；将两个精确原名交 Root，供同一第 2 项后续批次加入，从 V9 最低 20 基准至少提高到 22。
- [ ] 后续兼容批次的三包 check 和两条原用例真实执行通过，测试名、匹配数量及输入守卫核对成立。
- [ ] 原 Runtime / Editor 全部性能 producer、规模矩阵、配置和数值门槛达标。
- [ ] 原计划要求的真实产品、Jenkins 和里程碑验收完成。

本次未运行 Cargo、原生测试或性能，未修改 validation manifest，未查询协调器实时状态，未提交或推送。上轮 batch04 的首项 `101`、其余 14 项未执行和零测试 / 性能执行仍保留。

[采用收据](../../../../../.codex/outbox/runtime-dynamic-session-test-wake-construction-adoption-v1/adoption-receipt.json) SHA256 `9d86388f44e6bdce606d3cb0900145584d10358b129ca0c06b6d05e3d7a2639d`；[原筛选交集与 scoped 证据](../../../../../.codex/outbox/runtime-dynamic-session-test-wake-construction-adoption-v1/scope-and-original-filter-receipt.json) SHA256 `ec9c5e282e380a78619228b7f32d90a179ef98a3b807cca619dd40ee1baf9b2f`。文件采用后的 LF SHA256 为 `1e036bffd0aacd0c452639f07cc1d63dc2f7fd962a2d0b40c1eaa1d8a26bc054`。
