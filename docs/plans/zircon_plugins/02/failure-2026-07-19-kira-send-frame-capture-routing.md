---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: kira-send-frame-capture-routing
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_plugins/02-sound.md
origin_child_dir: docs/plans/zircon_runtime/render/01
fixing_child_dir: docs/plans/zircon_plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_compile.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_compile/routes.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/graph.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/graph/transaction.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/graph/routing.rs
tests:
  - post_effect_send_obeys_target_bus_gain_mute_and_parent_gain
  - master_track_gain_is_applied_once_to_direct_and_send_paths
  - active_graph_sync_updates_the_rendered_send_for_parent_gain_changes
---

# Plugins02: Kira send frame-capture routing current-source RED

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 来源执行切片：Render01 frame-capture route gate
- 修复责任计划：`docs/plans/zircon_plugins/02-sound.md`
- 交接原因：Sound Kira graph compile/route installation boundary 是三项 send routing 失败的最低 owner。

## 失败现象与复现证据

- Managed job `99687c8d6c584399aa727b09e121cdc1`, run `dc0d773d6c1e4b8b9b2b610114f7867c`, completed/released with exit `101`.
- Exact frame-capture gate executed `4` tests: `1 passed / 3 failed / 325 filtered`.
- The three failures show that post-effect send contribution was absent, master gain did not scale direct/send paths exactly once, and active graph resync did not update the rendered send after parent-gain changes.
- Build completed in `7.62s`; tests completed in `0.06s`. This is accepted as the Sound RED baseline and is not a Render-owned failure.

## 最低共享层根因

The failure belongs to the Sound Kira graph compile/route installation boundary. Render01/F2 and Shader06 must not patch or absorb these paths.

## 架构修复验收

- Fresh canonical Rust 1.94.1 focused route GREEN on the exact current source.
- Fresh plugin broad/product GREEN and current lock evidence.
- Independent review Critical/Important `0/0`.
- Coordinator-managed atomic milestone commit with immutable SHA and shared staged count `0`.

## 禁止临时方案

Render01/Shader06 不得吸收 Sound 路由修复，不得弱化三项 frame-capture 失败断言，也不得用非 current-source 产物替代 focused gate。

## 修复结果与回传

This record remains `open` until all current-source acceptance evidence is recorded.
