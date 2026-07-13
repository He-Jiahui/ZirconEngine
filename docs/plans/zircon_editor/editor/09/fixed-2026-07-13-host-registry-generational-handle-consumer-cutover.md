---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: host-registry-generational-handle-consumer-cutover
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/runtime/13
related_code:
  - zircon_runtime/src/script/vm/handles.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_editor/src/ui/host/host_capability_bridge.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
tests:
  - cargo test -p zircon_runtime --lib script::vm::host::host_registry --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
resolved_at: 2026-07-13
---


# Runtime 13：HostRegistry generational handle 硬切未迁移 Editor consumers

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-13 | Editor09 M1 全量 Windows 门已证明 Runtime13 的 generational `HostHandle` owner 已落地，但 Editor capability bridge 仍把可失败注册结果当成 `HostHandle`，Manager 测试仍调用已删除的 `capability(...)`。失败已下沉到 Runtime13，不在 Editor09 资产域增加 unwrap、旧查询 alias 或兼容 facade。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：Editor09 M1 完整 Windows lib-test 验收
- 修复责任计划：`docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md`
- 交接原因：最低共享 owner 是 Runtime13 明确负责的 VM host handle、host capability registry 与
  marshalling/capability 语义。Editor09 只是第一个暴露未原子迁移 consumer 的上行计划，不应在资产
  管理切片恢复旧 `capability` API 或把新的 typed registration failure 静默 unwrap。

## 失败现象与复现证据

Windows 受管 job `5b3fce449e73469a857795e6027ef9f3` 执行：

```text
cargo test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
```

在当前源码编译 `zircon_editor` lib-test 时自然退出 101，产生两处确定错误：

- `zircon_editor/src/ui/host/host_capability_bridge.rs:55`：
  `register_capability(...)` 已返回 `Result<HostHandle, HostRegistryError>`，consumer 仍把结果直接写入
  `EditorHostCapabilityHandle.handle: HostHandle`，得到 E0308。
- `zircon_editor/src/tests/host/manager/minimal_host_contract.rs:50`：测试仍调用已删除的
  `HostRegistry::capability(handle)`，而当前 canonical 查询入口是 `resolve(handle)`，得到 E0599。

完整证据日志：`.codex/tmp/editor09-m1-full-lib-test-20260713.log`。该失败发生在测试执行前，不能计作
Editor09 M1 完整通过。

## 最低共享层根因

`HostRegistry` 已从单调 raw id + `HashMap` 硬切为 index/generation slot owner：注册可因 slot index
耗尽返回 typed error，查询/撤销必须验证 generation，stale handle 不能重新变成有效对象。Runtime13
生产 owner 与 module-local tests 已迁到 `register_capability -> Result`、`resolve`、`revoke`，但两个 Editor
consumer 没有随公开合同原子迁移，形成共享 API hard-cut 中间态。

## 架构修复验收

- Runtime13 明确并保持 `register_capability -> Result<HostHandle, HostRegistryError>`、`resolve`、
  `revoke` 为唯一合同；host registry focused tests 覆盖 dead/stale/vacant/generation-exhaustion 语义。
- Editor VM capability bridge 对每次注册的 typed failure 作显式诊断投影，只记录成功 handle；不得 panic、
  丢弃失败或伪造 handle。Manager 测试使用 canonical `resolve(handle)` 验证 record。
- 全仓无 `HostRegistry::capability` 旧调用，也不新增同名 alias、兼容 trait 或 facade re-export。
- 原始 Editor09 完整命令重新编译并自然进入测试执行；其最终 summary 仍由 Editor09 M1 验收负责。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止在 Editor bridge 直接 `.unwrap()`/`.expect()` typed registration error，禁止恢复
  `capability(...) -> Option<_>`，禁止把 generation-aware handle 降回 raw 单调 id。

## 修复结果与回传

- 根因：Runtime13 HostRegistry had completed the generational Result/resolve hard cut, but the Editor capability bridge and Manager test consumer were not migrated atomically.
- 架构修复：The Editor bridge now matches typed registration results, records diagnostics and only stores successful handles; the Manager contract resolves handles through the canonical generation-aware resolve API, with no capability alias or unwrap fallback.
- 验证：Editor09 full Windows lib-test job e81ed19d256f40c28ddb2437e9a18460 compiled the current editor test binary successfully, eliminating the original E0308/E0599, entered test execution, and passed tests::host::manager::minimal_host_contract::editor_manager_registers_minimal_host_capabilities_as_vm_handles_when_script_is_available.
- 回传：Runtime13 generational HostRegistry consumer cutover is verified fixed and returned to Editor09; the later full-gate hang is independently owned by Editor15.
