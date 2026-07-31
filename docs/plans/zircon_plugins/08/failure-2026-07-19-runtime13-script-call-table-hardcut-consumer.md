---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: runtime13-script-call-table-hardcut-consumer
origin_plan: docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
fixing_plan: docs/plans/zircon_plugins/08-zr-vm.md
origin_child_dir: docs/plans/zircon_runtime/runtime/13
fixing_child_dir: docs/plans/zircon_plugins/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --lib --features backend-zr-vm zr_vm_backend_has_one_plugin_owned_dense_production_path --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --lib --features backend-zr-vm --locked --jobs 1 -- --nocapture --test-threads=1
---

# Plugins 08: migrate the ZrVM consumer to the generation-owned ScriptCallTable hard cut

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md`
- 来源执行切片：Runtime13 generation-owned `ScriptCallTable` cache and borrowed lookup hard cut
- 修复责任计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 交接原因：Runtime13 owns the neutral host-export registry and immutable call-table contract, while Plugin08 owns the concrete `backend-zr-vm` consumer and its native-function lifetime boundary. The lowest real consumer correction therefore belongs to Plugin08.

## 失败现象与复现证据

Runtime13 now publishes one immutable `ScriptCallTable` per registry generation. `HostExportRegistry::script_call_table` directly returns that cached table snapshot rather than `Result<ScriptCallTable, VmError>`, and `ScriptCallTable::resolve` returns `Option<&ScriptCallSite>` so lookups do not clone callbacks on the hot path.

At handoff creation, Plugin08 source still encoded the removed API in `zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs`:

- `let call_table = host.host_exports.script_call_table()?;` applied `?` to a direct value.
- The result of `.resolve(&module.descriptor.name, &function.name)` was borrowed, while `build_native_function` intentionally takes an owned `ScriptCallSite` because the registered ZrVM callback closure outlives the registration loop.
- `zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs` source-guarded the obsolete exact `script_call_table()?` spelling.

Independent Runtime13 review reported Critical 0 / Important 1 / Minor 1 and rejected an exact-three-file Runtime13 commit until this consumer is migrated atomically. The Runtime13 callback re-entry regression now covers the minor lock-release concern. No Plugin08 feature-pass or milestone acceptance is claimed by this handoff.

## 最低共享层根因

The neutral Runtime13 owner correctly changed the contract from fallible table reconstruction plus owned lookup to a generation-owned immutable snapshot plus borrowed lookup. Plugin08 has not moved its concrete ownership boundary with that hard cut. A native callback must own exactly one cloned call site at registration time; retaining the obsolete `Result` call or restoring owned `resolve` would reintroduce the per-lookup allocation and duplicated validation owner that Runtime13 removes.

## 架构修复验收

- In `host_modules.rs`, remove the obsolete `?` from `script_call_table()`.
- Resolve each host callback through the borrowed table and call `.cloned()` exactly once before passing the call site into `build_native_function`.
- Update the Plugin08 source guard to require the direct table call and the single registration-boundary clone, while continuing to reject `call_with_capabilities` or registry lookup inside the native callback.
- Run the focused `zr_vm_backend_has_one_plugin_owned_dense_production_path` feature test with raw one-test evidence.
- Run the Plugin08 `backend-zr-vm` lib suite from a source-bound managed command, then return the fixed artifact so Runtime13 can rerun its call-table gate and final independent review.

## 禁止临时方案

- Do not restore `Result<ScriptCallTable, VmError>` or make `ScriptCallTable::resolve` return an owned call site.
- Do not add a compatibility helper, deprecated alias, cfg-gated old path, duplicate callback map, or callback-time registry lookup.
- Do not weaken or delete the Plugin08 dense-production-path source guard.
- Do not absorb unrelated current changes in `registration.rs` or other Plugin08 paths into this migration.

## 修复结果与回传

- 2026-07-22 Plugin08 candidate：`host_modules.rs` 已删除 obsolete `?`，borrowed `resolve` 后在 native-function registration boundary 执行唯一一次 `.cloned()`；callback closure 继续只消费 owned `ScriptCallSite`，不进行 registry lookup。source guard 先以 direct-call 缺失、obsolete call 存在、clone count 0 得到 RED，再以 direct call / resolve / single clone / registration 的严格顺序得到静态 GREEN；`registration.rs` 既有 Scene constant 外部改动保持原样。scoped rustfmt、source contract 与 diff-check 已通过，managed Plugin08 feature gates 和 fixed return 仍 pending。
- 2026-07-22 managed gate：reservation `960b69f5b7024610a022acdefdb17c59`、job `a3dbb677be3846ad9538e1edffc2dfaa` / run `afac3a9afdb0400489163c56f6bd7232` 已自然 released `exit 101`、无存活 PID。Cargo 在测试执行前发现 `zircon_plugins/Cargo.lock` 需要更新，并因 `--locked` 正确拒绝修改；raw stdout 为空，`zr_vm_backend_has_one_plugin_owned_dense_production_path` 未执行。该 lockfile 不在本 exact-seven consumer 修复范围内，不得通过删除 `--locked`、手工吸收其他 manifest 变更或引用本 job 关闭 handoff；当前仍无 feature pass/fixed return。

Open state: `待修复`; no pass is claimed.
