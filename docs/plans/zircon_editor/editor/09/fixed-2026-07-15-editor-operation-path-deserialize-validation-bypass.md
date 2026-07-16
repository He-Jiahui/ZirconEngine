---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
resolved_at: 2026-07-15
summary_slug: editor-operation-path-deserialize-validation-bypass
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_editor/editor/08
related_code:
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/asset/toolkit_route.rs
  - zircon_editor/src/tests/editor_event/runtime/registry.rs
tests:
  - cargo test -p zircon_editor --lib --locked tests::editor_event::runtime::registry -- --test-threads=1
---


# Editor08：EditorOperationPath 反序列化绕过 canonical parse

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-14 | Editor09 typed toolkit route 审查发现 `EditorOperationPath` 直接 derive `Deserialize`，JSON newtype 可绕过 `EditorOperationPath::parse` 的三段、lowercase 与字符集校验。Editor09 会在自身 route wire decoder 中显式调用 parse，避免本功能泄漏；canonical type 的全局 serde 不变量归 Editor08 命令/操作 owner，禁止在每个 consumer 永久复制验证。 |
| `OPEN / RED 已复现` | 2026-07-14 | 独立 harness 直接编译当前 `editor_operation.rs` 与 `toolkit_route.rs`，输入 `open_operation = "Invalid Operation"` 后 `serde_json::from_value::<AssetToolkitOpenRoute>` 意外成功，期望拒绝测试为 0 passed / 1 failed；日志 `.codex/tmp/editor09-toolkit-route-invalid-operation-red-20260714.log`。这证明不是仅靠源码推断。 |
| `FIXED / canonical serde 已回传` | 2026-07-15 | `EditorOperationPath` 手写 `Deserialize` 并调用唯一 `parse`；`AssetToolkitOpenRoute` 删除局部 `WireRoute<String>` 验证，改为对 typed id 普通 derive 且保留 `deny_unknown_fields`。受管 lower-layer job `8cab7dad3c4d4c5a9932d9588df1b617` 为 1/1；current Cargo-built `zircon_editor-57c0b3f1608553a4.exe` 的 route 为 3/3、registry/command/extension/Remote/CLI 为 14/14；`cargo fmt -p zircon_editor -- --check` 与 scoped `git diff --check` 通过。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1.3 `AssetToolkitOpenRoute` serde hard cut
- 修复责任计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 交接原因：Editor08 明确拥有 `core/editor_operation.rs`、统一 operation id 空间与命令 wire contract；
  Editor09 只拥有资产 route，不能改变全部 CLI/plugin/registry 反序列化行为而不跑 Editor08 矩阵。

## 失败现象与复现证据

当前类型为：

```rust
#[derive(..., Serialize, Deserialize)]
pub struct EditorOperationPath(String);
```

直接构造调用 `parse` 会拒绝 `weather.refresh`、uppercase、连字符与空格；serde newtype decode 则直接写入
inner `String`，不会调用 `parse`。因此 persisted workspace、plugin contribution、CLI/control DTO 或其他
JSON consumer 可以获得无法由公开构造器创建的 `EditorOperationPath`。

直接复现日志 `.codex/tmp/editor09-toolkit-route-invalid-operation-red-20260714.log`：
`assertion failed: decoded.is_err()`，0/1、0.01 秒。临时 harness 与 executable 已删除。

## 最低共享层根因

验证只存在于构造方法，不存在于类型的 wire boundary。`AssetTypeId` 已使用手写 Deserialize 调回
`parse`，`EditorOperationPath` 没有遵循相同 typed-id 合同。每个上层 route 自行验证只能局部止血，会
形成重复验证与漏网 consumer。

## 架构修复验收

- `EditorOperationPath` 的 Deserialize 必须调用 canonical `parse`，错误通过 `de::Error::custom` 保留。
- 添加 serde 正反矩阵：合法 path roundtrip；少于三段、uppercase、连字符、空格与空 segment 均拒绝。
- command registry、extension/plugin contribution、CLI/control DTO 既有 focused tests 自然通过。
- Editor09 局部 wire decoder 可在 canonical type 修复回传后收敛，不能保留两套长期验证真源。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止只在 toolkit route、plugin loader 或 CLI 各写一份字符串规则；禁止对旧非法值静默规范化。

## 修复结果与回传

- 根因：Derived Deserialize wrote the private String directly and bypassed EditorOperationPath::parse.
- 架构修复：EditorOperationPath now owns handwritten canonical serde; AssetToolkitOpenRoute derives against the typed id and keeps no duplicate parser.
- 验证：Managed lower-layer 1/1; current Cargo-built toolkit route 3/3 and registry/command/extension/Remote/CLI 14/14; fmt and scoped diff checks passed.
- 回传：Canonical operation-path wire validation is fixed; Editor09 typed toolkit route can resume.
