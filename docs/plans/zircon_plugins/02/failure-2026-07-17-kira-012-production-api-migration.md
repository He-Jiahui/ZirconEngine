---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: kira-012-production-api-migration
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_workflow_node: M1
fixing_plan: docs/plans/zircon_plugins/02-sound.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/sound/runtime/src/kira_bridge/device.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/playback_data.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/status.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/backend.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/configuration.rs
tests:
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked
---

# Plugins02：Kira 0.12 生产 API 迁移未闭合

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：`M1`；Sound lockfile 修复后的上游恢复门禁仍待执行。
- 修复责任计划：`docs/plans/zircon_plugins/02-sound.md`
- 交接原因：Plugins02 的 Kira 0.12 生产适配层在编译前失败，Editor02 无法取得有效的 scene 门禁结果。

## 失败现象与复现证据

受管 job `63a9a9d2f7794d488b61b530e5436fd2` / run
`7c05366c20554ea7a1a8f9ae68fb5faa` 执行：

```powershell
cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked
```

该作业于 2026-07-17 05:49 +08 terminal/released，exit `101`、无 live PID。
stdout 为 0 bytes，测试二进制没有生成，实际 test count 为 0。不可变 stderr 报告
`18` 个编译错误（另有 warnings），包括：

- `kira_bridge/device.rs` 使用的 CPAL `DeviceTrait`、`SampleRate` 构造和设备枚举接口与 Kira 0.12 re-export 不匹配；
- `kira_bridge/manager.rs` 与 `playback_data.rs` 将 `f32` 直接传给新的 `Value<PlaybackRate>` API；
- output-device lifecycle/status/backend/configuration 调用尚未恢复的 canonical runtime-state unavailable 状态方法，继发 E0282。

## 最低共享层根因

锁文件已把 Kira 0.12 的依赖图固定下来，但生产 adapter 仍假设旧 CPAL/Kira 的松类型 API。
设备选择、播放速率和 unavailable backend 状态是同一 Sound runtime 边界；只修一处或恢复旧依赖都会留下不一致的运行时契约。

## 架构修复验收

- 以 Kira 0.12 public API 重建设备枚举、`StreamConfig`、播放速率和值类型转换，不依赖退役的直接 CPAL 兼容路径。
- 在 `SoundOutputDeviceRuntimeState` 的 canonical lifecycle owner 恢复 unavailable backend 的读写语义，并由 status/backend/configuration 共用。
- 先以 TDD 覆盖 Kira 0.12 CPAL device、PlaybackRate 和 unavailable-backend state 的真实 API contract；随后重新运行原始受管 plugin command，必须实际编译并给出 test count。
- 仅在 focused 与 plugin broad gate green、独立 review、immutable managed commit 和 failure→fixed return 后，才能重新验证三份 lock/manifest 文件并回到 Editor02 原始 gate。

## 禁止临时方案

- 不得移除 `--locked`、降级 Kira、恢复直接 `cpal`/`cpal-backend` 依赖，或把 compile failure 记为 metadata green。
- 不得由 Editor02、Render01、Shader06 修补 Sound source，或让 F2/PBR 队列越过这个失败。

## 修复结果与回传

Open state: `待 Plugins02 完成 Kira 0.12 support-first 适配、fresh managed plugin gate、独立审查和 source/lock closure 后回传`。
