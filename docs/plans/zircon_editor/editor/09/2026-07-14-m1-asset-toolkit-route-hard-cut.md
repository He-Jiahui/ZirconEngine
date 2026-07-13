---
status: completed
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
recorded_at: 2026-07-14
milestone: M1
slice: 1.3-route-correction
related_code:
  - zircon_editor/src/core/asset/toolkit_route.rs
  - zircon_editor/src/ui/host/editor_event_execution/asset_event.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/
  - zircon_editor/src/ui/host/asset_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
tests:
  - zircon_editor/src/tests/editor_asset_type_registry/toolkit_route.rs
  - zircon_editor/src/tests/editor_event/runtime/integration.rs
---

# Editor09 M1.3 资产工具包 typed route/source resolution 硬切

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|---|---|
| M1 | 1.3 route correction | `IN_PROGRESS` | 2026-07-14 | 完成架构根因定位并写入 RED：generic `OpenAsset` 已要求 catalog indexed type，但 view payload 仍保存机器绝对路径，真实 `res://` locator 会被 animation session 当作 OS 路径读取。新增 `AssetToolkitOpenRoute` 合同测试后，Windows no-run 精确得到两项 E0432（新类型尚不存在），日志 `.codex/tmp/editor09-toolkit-route-red-20260714.log`。实现已落地 typed locator + operation route、ProjectManager source resolution、animation route 持久化及旧 `path/operation_id` payload 删除；格式与 scoped diff 已通过。Cargo GREEN 尚未取得：共享 target 自 00:44:47 被另一 Editor 视觉测试占用，本轮 10 分钟只等待 artifact lock；本行不声明编译或测试通过。 |
| M1 | 1.3 route core contract | `CORE GREEN / CARGO PENDING` | 2026-07-14 | 独立 `rustc --test` harness 直接包含当前 `editor_operation.rs` 与 `toolkit_route.rs`，route serde roundtrip 与旧 payload 拒绝 2 passed / 0 failed / 0.01s，日志 `.codex/tmp/editor09-toolkit-route-standalone-20260714.log`；临时源码与 executable 已删除。该结果只验收 core route，不替代完整 `zircon_editor` 编译或 host integration。 |
| M1 | 1.3 route deserialize validation | `CORE GREEN / CARGO PENDING` | 2026-07-14 | 新增 invalid `open_operation` RED 后，`AssetToolkitOpenRoute` 改为 `deny_unknown_fields` wire decoder 并显式调用 `EditorOperationPath::parse`。独立 harness 直接编译当前 route 与 operation 源码，roundtrip、旧 `{ path, operation_id }` 拒绝、非法 operation 拒绝共 3 passed / 0 failed / 0.02s，日志 `.codex/tmp/editor09-toolkit-route-standalone-green-20260714.log`；临时源码与 executable 已删除。canonical `EditorOperationPath` 派生 Deserialize 绕过 parse 的全局缺口仍由 Editor08 handoff 负责，本切片没有伪造兼容别名。 |
| M1 | 1.3 route Cargo testing stage | `COMPLETED` | 2026-07-14 | 仓库 validator 在 coordinator-managed Windows pool 编译出当前 `zircon_editor` lib-test binary；同一当前二进制中 `editor_asset_type_registry::toolkit_route` 3/3、indexed toolkit open 与 suffix rejection 2/2 通过，日志 `.codex/tmp/editor09-toolkit-route-cargo-green-20260714.log`、`.codex/tmp/editor09-toolkit-open-integration-green-20260714.log` 与 `.codex/tmp/editor09-editor07-toolkit-upward-focused-20260714.log`。完整 package run 编译成功后以 Cargo 101 且无 test summary 结束，继续归 Runtime11/Editor14 已有 full-harness 资源生命周期 handoff；该聚合失败不撤销本 route 的 5 项 focused GREEN，也不宣称 Editor09 整体 M1 完成。 |

### 最低共享层根因

M1.3 的 registry 分派只解决了“哪个 toolkit 打开资产”，没有完成“toolkit 如何持久化并恢复资产身份”。
generic view payload 使用 `{ path, operation_id }`，animation session 又把 `path` 直接交给
`std::path::Path`。测试传临时绝对路径时曾偶然可用，但 catalog 的 canonical identity 是 `AssetUri`；
生产 `res://...` 即使已通过索引与 registry 门，也不能作为本地文件读取。只给 Editor07 测试补目录
索引会隐藏真实生产缺口，因此 route/source resolution 必须先由 Editor09 修到最低共享层。

### 架构硬切

- `core::asset::AssetToolkitOpenRoute` 只持有 canonical `AssetUri` 与 typed
  `EditorOperationPath`，serde 使用 `asset_locator/open_operation`，并拒绝未知旧字段。
- `OpenAsset` 先解析 `AssetUri`，再用 typed locator 查询 catalog；无 scheme 的本地绝对路径在入口
  被拒绝，不再按 suffix 或物理路径推断 toolkit。
- generic view payload 只序列化 typed route，不再写入 `path` 或 `operation_id`。
- animation workspace 保存 typed route；restore 经当前 project 的 `ProjectManager::source_path_for_uri`
  解析实际 source path，sync/edit/save 始终保留 locator。保存后的 reimport 直接使用 locator，删除
  source-path 反推 identity 的第二路径。
- UI asset restore 接受 typed toolkit route 或当前 domain `UiAssetEditorRoute`；已删除 generic 旧
  `{ path: ... }` payload fallback。UI domain 自身的裸字符串 route 后续收束仍归 EditorUI05，不在本
  切片伪造兼容 wrapper。

### RED 与 GREEN

- RED：`cargo test -p zircon_editor --lib --locked --no-run --jobs 1` 在新增测试处得到两项
  `unresolved import crate::core::asset::AssetToolkitOpenRoute`，符合预期；不是既有 unrelated failure。
- 已完成：scoped `rustfmt --edition 2021` 与 `git diff --check`；generic/animation 生产范围扫描中旧
  `serializable_payload["path"]`、`get("path")`、`json!({ "path" ... })`、`operation_id` payload 为零命中。
- core GREEN：独立 harness 直接编译当前 route 与 operation 源码，最新 3/3 通过；这不是字符串模拟测试。
- 已闭合本 route RED：invalid `open_operation = "Invalid Operation"` 现由 route wire decoder 显式调用
  `EditorOperationPath::parse` 后拒绝。canonical type 自身的派生 Deserialize 绕过 parse 仍已交接 Editor08，
  本切片不跨 owner 修改全局类型。
- 治理复验：failure validator 为 83 artifacts / 0 errors；plan-output audit 仍仅报告既有 6 项
  Editor01、EditorUI01/10/11/index 与 Plugins05 归档问题，本记录没有新增违规，既有项继续由各自
  failure handoff 处理。
- Cargo GREEN：受管 Windows 当前 binary 中 route 3/3 与 indexed-open/suffix-rejection 2/2 通过；
  `zircon_editor` 完整 package 聚合运行仍由既有 full-harness 资源生命周期 handoff 管理，本记录不把
  无 summary 的 Cargo 101 扩大为 route 失败。
- 下游：Editor07 已消费本 route，开始把 animation fixtures 迁到真实 ProjectAuthority、catalog index、
  plugin registry capability 与 `res://` locator；其完成/回传仍由 Editor07 artifact 单独验收。

### 失败归属

- 共享 artifact lock 已通过 coordinator-managed testing stage 取得并释放；没有创建伪功能 handoff。
- Editor07 既有 [animation asset-open/index fixture failure](../07/failure-2026-07-13-animation-asset-open-index-fixture-cutover.md)
  继续保持 open。其修复必须消费本切片 typed route，不由 Editor09 跨 owner 改写全部 animation 行为测试。
