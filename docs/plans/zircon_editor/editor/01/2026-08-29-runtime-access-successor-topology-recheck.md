---
status: current-topology-rechecked
created_at: 2026-08-29
implementation_status: successor-boundary-guard-aligned
managed_validation_status: pending-product-lane
related_code:
  - zircon_editor/src/ui/host/editor_event_runtime_access/mod.rs
  - zircon_editor/src/ui/host/runtime_services.rs
  - zircon_editor/src/ui/retained_host/app/runtime_lease.rs
  - zircon_editor/src/ui/retained_host/app/asset_runtime_access.rs
  - zircon_editor/src/ui/retained_host/viewport/render_framework_access.rs
  - zircon_editor/src/tests/ui/boundary/runtime_service_access_cutover.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
---

# Editor01 runtime access successor topology recheck

## 结论

历史平面 owner `zircon_editor/src/ui/host/editor_event_runtime_access.rs` 已不存在，不能申请或复用该路径的旧 blob。当前 topology 已收束为 `editor_event_runtime_access/` 目录：`mod.rs` 只声明 asset、component、event、extension、input、settings、snapshot、status、workbench projection 等窄 owner，测试独立位于 `tests.rs`。

通用 runtime lifetime 没有重新泄漏到 workbench：`ui/host/runtime_services.rs` 以 `EditorHostRuntimeServices` 集中持有 `CoreWeak` 并发布 typed operations；retained-host 仅在 `runtime_lease.rs`、`asset_runtime_access.rs` 与 viewport `render_framework_access.rs` 保留已登记的窄 lifetime/access owner。`runtime_service_access_cutover.rs` 已拒绝 production UI 中的 `LevelSystem`、`ManagerResolver`，并以 owner allowlist 约束 `CoreHandle`/`CoreWeak`。

本轮发现 `host_cutover.rs` 仍静态要求已删除的 `create_runtime_level` 名称，而 current source 已硬切为 `prepare_authoring_world`。守卫已同步到 current typed API；没有恢复 alias、wrapper 或 legacy fallback。

## HighlightSet current-source 复核

历史 failure 中“gateway 未实现”的描述已经过时。current source 已包含 `submit_highlight_set` gateway contract/handle/in-process/session 路径、V6 ABI/runtime latest-value store、scene viewport 的 `EditorRuntimeHighlightSet` 构建以及 workbench frame 前提交。旧 `SelectionHighlightExtract` 与 `overlays.selection` production symbol 已不存在。

generation 复核未发现需要新增 editor-side counter：runtime store 只拒绝更小 generation，相同 generation 的 tint/display 变化仍会替换当前值；runtime/level replacement 也重置 store。因此本轮不建立第二套 generation owner，也不把未运行的 managed test 记为 fixed。原 failure 保持 open，直到 focused gateway/session/viewport 与产品 lane 获得终态证据。

## 验证边界

- current topology 与 symbol inventory：完成。
- `host_cutover.rs` current API 静态契约：已修正，scoped rustfmt/diff-check 待同批验证。
- managed Cargo：本轮 Editor09 focused product command 两次分别在 184.1 秒和 304.4 秒超时，未产生可归因诊断；Editor01 未据此声明 GREEN。
- 已知 workspace 级 `zircon_runtime_host` 的 `WorldQueryResult::TransformSnapshot` 穷尽匹配 failure 仍由其 owner 处理，不在本记录中修改。

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-29 00:53 +08:00 | `current-topology-rechecked / old-blob-rejected / successor-guard-aligned / managed-validation-pending` | 确认旧 `editor_event_runtime_access.rs` 已消失，inventory 当前目录化 event-runtime owner、typed `runtime_services` 与 retained-host 窄 lifetime owner；未请求归档 blob transfer。修正 `host_cutover.rs` 对已删除 `create_runtime_level` 的陈旧断言，使其要求 current `prepare_authoring_world`。复核 HighlightSet 已贯通 gateway/ABI/runtime/editor consumer，且同 generation replacement 语义成立，未新增重复 counter 或 overlay 通道。尚无 managed Cargo GREEN，原 HighlightSet failure 暂不关闭。 |
