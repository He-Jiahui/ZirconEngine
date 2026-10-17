# Editor06：插件当前实例的退役清理进度

状态：`source_reviewed / native_performance_pending`。核心代码与决策为 gpt-6.1-sol max，独立静态复核为 gpt-6-luna max；对应[Editor06 原计划](../06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md)的生命周期唯一 authority，本条不关闭其他 P0/P1 项。

CR-EDITOR-SERVICES-0002 的原因是用注册报告的历史 successful stages 判断本轮清理是否已完成。插件 Active→Disabled→Active 后替换，会把上一轮 Disabled 当作本轮结果；同一 Rust 实例刷新目录时，如果丢失历史摘要，还会漏掉 Faulted owner 的退役资格。历史报告继续承担原诊断合同，当前 owner 清理进度单独由 Catalog 维护。

真实 Enabled 尝试在回调前重置当前进度，包括失败尝试。只有退役路径调用专用 outcome wrapper，才按真实 Disabled/Unloaded 结果更新进度；public targeted/broadcast Unloaded 成功或失败都只写原历史报告，保留成功 state 不变、失败 Faulted 的合同，不得设置或撤销退役进度。Loaded/Enabled/Disabled 仍是管理器保留阶段，set_enabled(false) 状态与失败重试门槛保留。退役资格仍按原 Active/Revoking 或有历史 Enabled 成功的 Faulted 判断。

同 Rust Arc 的目录发布只延续 lifecycle report、成功/失败摘要与当前进度，保留候选的新 metadata、contributions 和 diagnostics。新 native 同ID的 project report 替换会清除被删除 project IDs 的进度，`None,None` 仍表示新宿主；中断退役回发布 previous clone 时保留旧宿主进度，使已成功 Disabled/Unloaded 不重复、失败阶段继续重试。

| 源码 | 字节数 | SHA256 |
| --- | ---: | --- |
| `zircon_editor/src/core/plugin/catalog.rs` | 15192 | `48bd3682422aa517fbf07b39ce71dcee2cd1f111e9b15e5b27d83ae1dc90e578` |
| `zircon_editor/src/core/plugin/manager/lifecycle_replacement.rs` | 4762 | `f2661142327c4126cbc55dc3876a2407ac4e157d6bbbfb989706bdda17254127` |
| `zircon_editor/src/core/plugin/manager/publication.rs` | 4505 | `8fb7b79013f637d8705a3cf3007bc5fee3d56ff284816a233726f1ed6ed8c09a` |
| `zircon_editor/src/core/plugin/manager/tests/cases.rs` | 47214 | `f9e9fd21f2b2ff48c1e7bae8892085239f38d4755d13cef0dce3e58797d30f01` |

[补丁](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor-plugin-cleanup-epoch.patch)、[核心证据](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor-plugin-cleanup-epoch-evidence.json)和[scoped guard](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor-plugin-cleanup-epoch-static.json)与最终字节一致，格式/diff/guard 实际退出0，独立静态审查无阻断。公开 registration 历史结构、B14 的失败 Unloaded 重试 fixture 和原有生命周期断言保留。新增真实 ordinary 回归：

- `core::plugin::manager::tests::replacement_retires_reenabled_instance_in_current_cycle`
- `core::plugin::manager::tests::replacement_retries_only_current_cycle_cleanup_failures`
- `core::plugin::manager::tests::reenable_attempt_starts_fresh_cleanup_even_when_enabled_fails`
- `core::plugin::manager::tests::same_plugin_catalog_refresh_preserves_lifecycle_authority`
- `core::plugin::manager::tests::native_cleanup_progress_survives_interrupted_replacement_retries`
- `core::plugin::manager::tests::ordinary_unloaded_notifications_do_not_complete_retirement`
- `core::plugin::manager::tests::ordinary_unloaded_failure_preserves_completed_retirement_progress`

manager namespace最终共 41 ordinary、0 ignored。B14 已有效选择其中 1 个，后续合批 ordinary floor 仅按实际新增覆盖去重；ignored producer 不计 ordinary，Windows/native-host feature reachability 必须按实际编译回执核验。

- [x] 当前 owner 清理进度与历史报告分离，真实 Enabled 重置、退役 outcome 更新。
- [x] public Unloaded 通知成功/失败不能冒充或撤销真实退役进度。
- [x] same Arc 生命周期 authority、新候选 metadata 与 native 新旧宿主边界保留。
- [x] 原 retry fixture、历史断言、限定4源哈希/格式/diff/guard 和独立静态审查完成。
- [ ] 后续合批 exact-source 原生回执、全名/计数与 full-input seal 通过。
- [ ] 全部适用原性能门槛及资源矩阵通过。
- [ ] 依据实际回执关闭历史 BUG 及整体验收。

本修复没有宣称已测得 P95/CPU/RSS 改善，也不将静态复核或文件交接当作原生验收。没有本地 build/cache/原生夹具或源码副本；保留必要证据，实际释放0B。提交后继续后续功能修复，不等待或持续监控编译。
