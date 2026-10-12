# Editor01：控制器测试适配 ServiceHandle（2026-10-09）

对应[原优化计划](../01-retained-ui-architecture-performance-review.md)与[功能完成列表](../../../astra/features/editor/2026-10-09-editor01-host-controller-test-completion-list.md)。本记录只覆盖测试 fixture 对现有生产 API 的适配。

## 实际失败与修复

12 命令原生批次 `runtime-editor-source-repair-batch-20261009-02` 的首条 `three-package-check` 以 Cargo `101` 结束，提交时绑定的源码未变化，后续 11 条命令均未调用。其中 `editor_world_sync_watch_map.rs` 有 12 个诊断：两处向控制器传入 `ServiceCallGuard<EditorManager>`，以及在构造结果 `Result` 上直接调用方法，产生 E0308/E0599；两处 E0614 随类型推断失败出现，需下一批真实编译确认。

已修复两处测试 fixture：通过 `CoreRuntime::resolve_manager_handle` 获取 `ServiceHandle<EditorManager>`；在临时 `enter()` guard 下 clone context，guard 在 state 初始化语句结束时释放；对 `EditorHostEventController::new` 的返回值显式 `expect` 成功。七个原测试、31 处原断言、消息合并和 barrier 顺序、pointer journal 验证保持；借用事件的 `*x/*y` 保留。生产服务准入与 owner API 维持其当前合同。

## 源码证据与异步验证

实际接入时间 `2026-10-09T06:18:07.966345+00:00`，rustfmt 与 scoped `git diff --check` 均为 `0`；控制配置、index、Cargo manifest/lock hash 未变。源码 `zircon_editor/tests/editor_world_sync_watch_map.rs` 的 LF hash 为 `3e7189f9d24ae6e9f8be2dc187f984ad75a65e59ccf71dea6562144e69fab6d6`，实际文件 raw hash 为 `732742d1f62a08a6b7c7a6fd6fd622355f6ed9e64ebafa6efcc8bd46b8259771`。

证据目录：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/`。`editor-host-contract-fix-v1/actual-canonical-apply.json` SHA256 为 `25b15564558d521ba9ba2b2602bdf2c1146739736182194b1fd3d14caad8becb`；独立审查 `editor-host-contract-core-review-v1/review.json` SHA256 为 `7f7ff80bc84e09b33472bdca3a0b3eb51953f6bde7ee27f67d8c3c82b0c394e8`，源码结论 accepted。

下一批需实际执行 `--test editor_world_sync_watch_map` 的七个 case，并与其他 runtime/editor 修复及原性能 producer 一批提交。13 命令草案尚未绑定与启动。现有失败收据完整保留；源码修复后需重新绑定。提交后继续功能修复候选，在实质代码或证据检查点一次校验终态。

## 完成列表

- [x] 追踪实际编译错误至 ServiceHandle/guard/Result fixture 合同。
- [x] 两处 fixture 修复已接入，独立审查、格式与差异检查通过。
- [ ] 实际编译确认 E0308/E0599/E0614 全部消失。
- [ ] 七个集成 case 全部实际匹配并通过。
- [ ] Editor01 原性能、产品、Jenkins 与里程碑验收完成。
