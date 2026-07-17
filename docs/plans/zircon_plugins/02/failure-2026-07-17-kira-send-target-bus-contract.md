---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: kira-send-target-bus-contract
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_workflow_node: M1
fixing_plan: docs/plans/zircon_plugins/02-sound.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/sound/runtime/src/kira_bridge/manager.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_compile.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge
tests:
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked
---

# Sound02：Kira send 未保持目标 bus 契约

## 来源执行者

- 来源计划：docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
- 来源执行者：plugins02-sound-m1-kira-core-20260717
- 来源执行切片：M1 上游恢复门禁中的 Sound M1 plugin gate
- 修复责任计划：docs/plans/zircon_plugins/02-sound.md
- 交接原因：Kira send 的 target bus 映射是 Sound M1 graph compiler 的最低共享职责，Editor02
  不拥有或修改 Sound runtime 路径。

## 失败现象与复现证据

独立审查当前 Kira 0.12.2 实现后确认，`KiraEngine::ensure_send_tracks` 为每个
`SoundTrackSend.target` 创建独立 `SendTrack`，而同一 `SoundTrackId` 的普通 Track 仍被独立创建。
source Track 直接以 `TrackBuilder::with_send` 接入 SendTrack。

受管 focused gate `e434d02e748b45c59ff97cf534212cbb` / run
`4cc98aa314dc4f07bb08455f0ef84c67` 为 22/0，full gate
`a33b61d0341f4b7aae19c8f4dd1f584f` / run `63148e97e9eb487e95f1bfbec9872230`
为 270/0；两者均未覆盖实际 send 输出，因此不能作为该 routing 契约的通过证据。

## 最低共享层根因

Kira SendTrack 的输入在自身音量/效果后直接混入输出；它不会经过目标普通 Track 的 gain、mute
或 parent。因此 `Music -> Aux` 的 send 会绕过 Aux bus 的 M1 可用控制，违反
`SoundTrackSend.target` 的目标 bus 契约。先前自研实现的实际路由增益测试在 hard-cutover 中被移除，
新的 DTO/diff 测试没有覆盖这一音频语义。

## 架构修复验收

- 对每个 logical send target，Kira SendTrack 必须应用该目标与所有祖先 Track 的有效 M1
  gain/mute；图同步时任一相关控制变化都必须重新同步该有效值。
- M1 不声称 Kira 尚不能表达的目标 bus effects / pre-effect route：它们继续由已有 M2
  surface rejection 拦截，不得静默绕过。
- 使用可捕获帧的 Kira test backend 证明 source send gain、target gain/mute 和 parent gain 都影响
  实际输出；不得只检查 DTO、track count 或 diff。
- 修复后重新执行 focused routing gate 与完整 canonical plugin gate，并完成独立审查后再 return
  `fixed-2026-07-17-kira-send-target-bus-contract.md` 给来源 child 目录。

## 禁止临时方案

- 不得删除或 `cfg` 跳过 send 路由测试，不得把 `SoundTrackSend` 全部标为 unsupported 以回避
  M1-T2 映射责任。
- 不得恢复自研音频渲染器、复制 source 音频或由下游 F2/Shader06 变更 Sound 路径。

## 修复结果与回传

Open state: `待 Kira logical target-bus gain/mute/parent adapter、frame-capture regression 与 fresh gates 完成。`
