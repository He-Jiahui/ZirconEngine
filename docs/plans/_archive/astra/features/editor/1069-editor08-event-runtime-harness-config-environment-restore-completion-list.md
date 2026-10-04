# Editor1069：EventRuntimeHarness 配置环境恢复

状态：公共支持层修复已应用并登记；managed Editor 测试、完整环境隔离及产品性能验收未完成。

优化记录：`docs/plans/optimize/zircon_editor/08/2026-09-30-event-runtime-harness-config-environment-restore.md`。

## 完成列表

- [x] 定位公共 Harness 丢失原 `ZIRCON_CONFIG_PATH`、初始化异常后残留临时值的问题。
- [x] 以 `Option<OsString>` RAII guard 恢复原值或缺省状态，保留原恢复边界、调用方锁和真实项目夹具；完成静态复核。
- [x] 重新核对 archived 所有者、当前字节和 epoch，取得单路径 live lease；应用候选，通过 rustfmt/scoped diff check 并登记源码归属。
- [ ] 在分组 managed Editor 验证中通过新增原值/缺省值、展开恢复与真实 Harness 构造回归。
- [ ] 合入独立 keymap 和两个场景视口实际环境锁修复，并通过完整 Editor 正常并发测试。
- [ ] 达到主计划真实产品延迟、分配和 RSS 门槛。

源码：`zircon_editor/src/tests/editor_event/support.rs`。应用前 SHA `24fb7736ceae18eafbb2481fa711552ffc8146c57bbe857fcc8ce1fe2214e796`；应用后 SHA `5e733822a84959278e04fa3a4ac206dd2e901e1436234f8690ca3e39f71f5c1f`。

实际应用日志 `.codex/state/session-coordinator/async-validation-batches/2026-09-30-editor-event-harness-config-environment-restore-guarded-apply.json`，SHA `d7810d555587a1f86a3f85112f0fa130dbb976e7222a1737168883aa0caf2a8d`。最终源码复核 SHA `0a141f1e36943af8f754a75a6cf71a1242ed7d92bdd4201db0fde31d9e289474`。源码已落地不代表 Cargo/typecheck、测试或性能通过；其他三个环境锁候选依然未应用。
