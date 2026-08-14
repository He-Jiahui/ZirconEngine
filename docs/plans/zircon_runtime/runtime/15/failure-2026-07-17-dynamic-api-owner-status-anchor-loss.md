---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: dynamic-api-owner-status-anchor-loss
origin_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/runtime/10
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - docs/plans/zircon_runtime/runtime/15/2026-07-19-dynamic-api-filter-plan-anchor-current-owner.md
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_session_profile.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_session_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_shader_prewarm_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_asset_dynamic_dynamic_api_vampire.rs
tests:
  - cargo test -p zircon_runtime --lib runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_15_dynamic_api_session_profile_is_child_owner --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_15_dynamic_api_session_registry_is_child_owner --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_15_dynamic_api_shader_prewarm_tests_are_child_owner --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_15_asset_dynamic_dynamic_api_vampire_guard_is_child_owner --locked --jobs 1 -- --test-threads=1
---

# Runtime15: Dynamic API owner status anchors were lost from current plans

## 产出记录与时间

| 时间 | 来源门禁 | 状态 | 失败证据 | 修复责任 |
| --- | --- | --- | --- | --- |
| 2026-07-17 | Runtime10 `dynamic_api` 上行门 | `待修复（open）` | Windows managed job `388b5dfbf30245328db1a66d0bb88978` / run `b1a01bc55a2d478d9067ac56a91e489f` 执行 112 项，94 passed / 8 failed / 10 ignored，exit 101。其中 5 项为 Runtime15 owner/status 镜像失败。独立 current-source 复审逐锚确认：dynamic-session lock-poison、profile owner、registry owner、shader-prewarm test 四组在 Runtime15 父计划分别缺 3/5、4/5、4/5、5/5，在 current runtime index 均缺 5/5；asset-dynamic dynamic-API vampire 组在父计划已 5/5，current index 缺 4/5。对应详细锚仍能在 Runtime15 numbered/status rows 或模块文档找到，说明生产 owner 未被证明失效，失效的是 parent/index 到 canonical child source 的可执行路由。 | Runtime15 重新建立 current parent/index 到 numbered child records 的单一可执行镜像，或同步迁移守卫到 canonical child owner；不得只补首个短路失败位置、复制生产实现、删除断言或恢复 archive 聚合为第二事实源。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 来源执行切片：Editor01 M2.3 selection-state hard cut 的 Runtime10 上行 `dynamic_api` 验证
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：五项失败都验证 Runtime15 的模块 owner、计划状态或总索引镜像；Runtime10 不应通过放宽结构守卫来关闭自己的上行门。

## 失败现象与复现证据

受管命令：

```text
cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 -- --test-threads=1
```

失败组：

- `runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry`：Runtime15 父计划缺 3/5，current index 缺 5/5 lock-poison owner/status anchors。
- `runtime_15_dynamic_api_session_profile_is_child_owner`：父计划缺 4/5，current index 缺 5/5 profile child-owner anchors。
- `runtime_15_dynamic_api_session_registry_is_child_owner`：父计划缺 4/5，current index 缺 5/5 registry child-owner anchors。
- `runtime_15_dynamic_api_shader_prewarm_tests_are_child_owner`：父计划缺 5/5，current index 缺 5/5 shader-prewarm test child-owner anchors。
- `runtime_15_asset_dynamic_dynamic_api_vampire_guard_is_child_owner`：父计划 5/5 anchors 已存在，current index 缺 vampire guard child-owner anchors 中 4/5。

上述五个测试都依次检查 parent 与 current index：前四个首次执行在 parent 断言处短路，vampire 测试通过 parent 后在 index 失败。修复必须按上述矩阵验证完整 required-anchor 集合，不能只补当前 panic 首报的位置。

同一门中的 Runtime10 selection-state event split 结构测试通过；本交接不覆盖独立的 Render01 F2 artifact URI 失败，也不覆盖 Runtime10 自己的两个 stale test-anchor 修复。

## 最低共享层根因

Runtime15 父计划和 runtime index 被压缩为路由/概览时，没有把既有结构守卫仍消费的 owner/status anchors 一并迁到当前 canonical child source。结果是生产模块和 numbered/status records 仍存在，但可执行文档镜像断链。该问题属于计划 owner 路由收敛，不属于 dynamic session 生产逻辑。

## 架构修复验收

- 五个聚焦结构测试全部通过，并继续锁定真实 child owner、文件预算、status，以及单一 current-evidence child record。
- 五组 required anchors 在 `docs/plans/zircon_runtime/runtime/15/2026-07-19-dynamic-api-filter-plan-anchor-current-owner.md` 中逐项通过；守卫必须删除对压缩 parent/index wording 的直接依赖，不得以“首个 panic 已消失”替代同一测试后续断言的验证。
- 每组 anchor 只有一个 canonical current owner；若守卫迁向 numbered child record，必须删除对 retired aggregate wording 的依赖，不得双写父计划与 archive。
- Runtime15 父计划继续保持概览职责，不重新堆叠完整历史正文；必要的 current status 可通过明确 child-record link/loader 读取。
- Runtime10 上行 `dynamic_api` 重跑时，这五项不再失败；其他功能 owner 的失败仍独立交接。

## 禁止临时方案

- 不得删除/忽略五个测试、缩短 required anchor 列表或从 `dynamic_api` filter 排除结构守卫。
- 不得把 archive aggregate 恢复为第二份 current truth，也不得复制 profile/registry/shader-prewarm 生产实现到计划或测试 helper。
- 不得把 Render01 F2 或 Runtime10 headless/status 锚失败混入本记录来制造一次性批量豁免。

## 修复结果与回传

- 来源交接独立复审：Critical / Important / Minor = 0 / 0 / 0；逐锚矩阵与测试短路顺序已按 current source 核实。该结论只确认 failure handoff 的准确性，不声明 Runtime15 修复完成。
- 2026-07-19 Runtime15 hard-cut candidate：新增单一 `current_evidence_owner` child record，五个守卫改为读取该记录并继续校验真实模块 owner/status/files；对 Runtime15 parent、runtime index、priority review 与 structure plans 的直接锚定已删除。静态 exact-anchor 矩阵 5/5、focused handoff schema 与 scoped diff-check 已通过。
- 首个 exact-seven validation copy 漏带新 child record，第二个副本在记录纠正前物化；对应预约 `e195382272174f4898f11632a15d2b87` 已在执行前释放。必须以纠正后的 fresh exact-seven copy/reservation 和原始五项 raw test 结果作为 Cargo 证据。
- 2026-07-22 current-source rebase candidate：dynamic session registry 的零行为 facade 是 `dynamic_api/session/registry/mod.rs`，全局存储与 handle/lifecycle owner 是 `registry/session_store.rs`，单 session 锁与 close barrier owner 是 `registry/session_slot.rs`；旧 `registry.rs` 与仅测试消费的 `lock_session` forwarding shim 已删除。测试不再扩大 registry/session 锁的生产可见性：registry poison 由 `#[cfg(test)]` owner helper 制造，session poison 通过 `with_session` 的真实 action-admission 路径制造。两项 registry 守卫验证 facade 零行为、`Arc<SessionSlot>`、poison recovery、wake 参数在 insert owner 内被实际消费、activity dispatch 与 destroy lifecycle；current child tuple 使用有界 tuple 切片并同步拒绝任意格式的旧 flat 路径。
- 2026-08-14 Runtime15 current-source hard cut：`dynamic_scene.rs` 删除三个 lock-poison guard 中 14 个未参与断言的计划、index、review、structure 和模块文档读取。首个 dynamic-API guard 继续 exact 读取唯一 current child record；spawn-task 与 parallel-executor guard 继续只验证实际生产锁恢复路径。已物理删除的 `plan_status` M4 row 不再作为 related source。
- 同日 `dynamic_api_session_profile.rs` 删除两个无断言消费者的 module/session 文档读取，继续 exact 验证唯一 current child tuple、parent/FFI/state/profile owner、moved-owner absence 与文件预算。

Open state: `resolving_failure`。fresh immutable review 与五项 managed current-source test 仍 pending；不声明 fixed/accepted。
