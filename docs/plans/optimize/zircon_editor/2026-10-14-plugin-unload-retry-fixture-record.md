# Editor：插件卸载失败重试夹具修复

状态：`source_reviewed / native_performance_pending`。[Plugin Manager 主计划](06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md)要求保留现有生命周期与 generation authority。当前修复为已有卸载失败重试用例的期望；主计划的产品 P0/P1 及更广激活周期问题继续开放。

[测试源码](../../../../zircon_editor/src/core/plugin/manager/tests/lifecycle_replacement.rs)中 CR-EDITOR-SERVICES-0003 的首次替换已成功 Disabled、失败 Unloaded。成功 stage 会被保留，重试应跳过已完成 Disabled、再次调用未完成 Unloaded。仅把最终事件向量第4项从 Disabled 改为 Unloaded，得到 Loaded, Enabled, Disabled, Unloaded, Unloaded。

首次 stage=Unloaded 的错误、Faulted 状态、被拒候选无事件、重试后 Active、Recovered 的 Loaded/Enabled/HotReloaded 断言全部保留；这同时验证失败步骤确实重试且成功步骤不重复。生产 cleanup/callback/state 逻辑没有修改。

| 源码 | 最终字节数 | SHA256 |
| --- | ---: | --- |
| `lifecycle_replacement.rs` | 18793 | `7bfc34238f40ec2d9c5238a983eb85f1705090fe1e01225a5cd8fd443c33556b` |

[补丁](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-editor-fixture-contract-repairs.patch)、[核心证据](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-editor-fixture-contract-repairs-evidence.json)、[静态 guard](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-editor-fixture-contract-repairs-static.json)已与最终字节核对；核心 fmt/diff、root guard 实际退出 0，独立 delta 静态复核无阻断。原文件仍为 6 ordinary、0 ignored，修复用例为：

- `core::plugin::manager::tests::lifecycle_replacement::replacement_retries_failed_unload_before_activating_candidate`

其与旧适用 Editor arm 的有效 ordinary 选择无重叠，第14批已有 Editor arm 下限 95→96；原所有 selected names、验证命令和性能门槛保留，禁止逐例编译验证。

- [x] 成功 Disabled 仅一次，失败 Unloaded 被调用两次的期望修正。
- [x] 首次失败隔离、旧实例保留、候选无事件与恢复激活断言完整保留。
- [x] source/packet/hash、静态检查和独立审查完成。
- [ ] 第14批 exact-source 原生执行、required name、匹配下限与 full-input seal 通过。
- [ ] 适用原性能门槛与完整资源清理证据通过。
- [ ] 根据实际回执关闭历史 CR-EDITOR-SERVICES-0003。

CR-EDITOR-SERVICES-0002 的跨激活周期成功标记问题独立调查；本记录不关闭它或主计划的其他缺口。未运行本地 native/product/performance，不创建构建/缓存/夹具、临时脚本或源码副本，实际释放 0B；保留必要交付证据。交接后继续功能修复，不等待或实时监控编译。
