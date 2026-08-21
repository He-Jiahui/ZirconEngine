---
handoff_kind: failure
status: open
created_at: 2026-08-16
summary_slug: editor-host-hub-handshake-config-visibility
origin_plan: docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
fixing_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
origin_child_dir: docs/plans/optimize/zircon_app/01
fixing_child_dir: docs/plans/zircon_editor/editor/16
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_app/src/entry/entry_runner/editor.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_app -NoDefaultFeatures -Features target-editor-host -Bin zircon_editor -SkipTest
  - cargo test -p zircon_app --lib --no-default-features --features target-editor-host --locked editor_host
---

# Editor16: editor-host Hub handshake config visibility

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md`
- 来源执行切片：App01 M2 editor-host 产品构建回归门槛
- 修复责任计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 交接原因：Hub session 参数、`EditorHostRunConfig` 构造和 retained-host 启动边界均由 Editor16 持有；App01 只是该公共宿主配置合同的调用方。

## 失败现象与复现证据

2026-08-16 执行以下 Windows 托管产品构建：

```powershell
./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_app -NoDefaultFeatures -Features target-editor-host -Bin zircon_editor -SkipTest
```

Hybrid GI 与此前的 project-generation artifact 修复已成功编译，构建随后在
`zircon_app/src/entry/entry_runner/editor.rs:282` 唯一失败：`E0624: method
with_hub_handshake is private`。调用方位于 `zircon_app` crate，而
`zircon_editor/src/ui/retained_host/run_config.rs:69` 将该 builder 声明为
`pub(crate)`，因此启用 `target-editor-host` 的真实产品组合无法编译。

## 最低共享层根因

Editor16 已把 Hub handshake 作为 `EditorHostRunConfig` 的跨 crate 启动输入，却仍以
crate-local 可见性发布对应 builder。类型的职责边界与 Rust 可见性边界不一致；这不是
App01 可在调用点绕开的局部错误。

## 架构修复验收

- 在 Editor16 所有权内形成唯一、文档化的公共配置入口，使 `zircon_app` 能为
  `target-editor-host` 注入 `(project_root, HubSessionToken)`；不得复制握手 DTO 或直接修改
  retained-host 内部字段。
- 添加或更新跨 crate 编译/行为测试，证明 Hub handshake 配置进入
  `EditorHostRunConfig::into_parts` 后保持 session 与 project root，且无 handshake 时行为不变。
- 上述 focused test 通过，并重新运行原始 `zircon_app` editor-host 产品构建至 `[OK]`。
- App01 随后继续运行 foreign-output release 性能验收和完整产品门槛。

## 禁止临时方案

- 禁止在 `zircon_app` 重建 `EditorHostRunConfig` 内部状态、复制 `HubEditorHandshake`，或增加
  feature-only/test-only bypass。
- 禁止移除 Hub handshake、吞掉 session，或通过降级 feature 集合让产品构建绕过该路径。
- 禁止保留第二个 builder/别名作为兼容层；公共入口必须是单一事实来源。

## 修复结果与回传

Open：`待修复`。当前只确认公共配置合同的可见性漂移，没有声明 editor-host 产品构建通过。

### 2026-08-18 current-source 前向修复

- `EditorHostRunConfig::with_hub_handshake` 已成为带文档的唯一公共 builder；应用组合根仍只传入
  `project_root + HubSessionToken`，没有暴露或复制 retained-host 内部 handshake DTO。
- `zircon_app` 的 `target-editor-host` 测试边界新增跨 crate 构造合同，直接从 App crate 调用公共
  builder，防止可见性再次退回 `pub(crate)`。
- 22 个统一 Runtime/Editor/App 覆盖文件的 `rustfmt +1.94.1 --check`、scoped
  `git diff --check` 和公共边界 source guard 已通过。
- source-bound 三包 compile 与 Runtime04 focused test 已合并到协调器副本
  `7f61b837316e4ea7a652511a290fbfd4`；本记录在其 Cargo terminal receipt 和原始 editor-host
  产品 gate 完成前继续保持 `open`，不提前声明回传。
