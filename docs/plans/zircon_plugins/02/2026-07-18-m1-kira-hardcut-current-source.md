---
related_code:
  - zircon_plugins/sound/runtime/Cargo.toml
  - zircon_plugins/sound/runtime/src/kira_bridge
  - zircon_plugins/sound/runtime/src/automation
  - zircon_plugins/sound/runtime/src/tests
  - zircon_plugins/sound/plugin.toml
  - Cargo.lock
  - zircon_plugins/Cargo.lock
tests:
  - post_effect_send_obeys_target_bus_gain_mute_and_parent_gain
  - master_track_gain_is_applied_once_to_direct_and_send_paths
  - active_graph_sync_updates_the_rendered_send_for_parent_gain_changes
  - direct_track_mute_automation_rejects_non_finite_input
  - direct_volume_priority_automation_rejects_non_finite_input
  - direct_chorus_voices_automation_rejects_non_finite_input_before_effect_lookup
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked --jobs 1 -- --nocapture --test-threads=1
---

Plan: docs/plans/zircon_plugins/02-sound.md
Milestone: M1
Status: completed
Files: ["docs/plans/performance/01/fixed-2026-07-18-kira-graph-sync-repeated-compilation.md","docs/plans/zircon_editor/editor/02/fixed-2026-07-18-kira-012-production-api-migration.md","docs/plans/zircon_editor/editor/02/fixed-2026-07-18-kira-send-target-bus-contract.md","docs/plans/zircon_editor/editor/02/fixed-2026-07-18-kira-test-module-topology-hardcut.md","docs/plans/zircon_editor/editor/02/fixed-2026-07-18-sound-kira-root-lockfile-drift.md","docs/plans/zircon_plugins/02/2026-07-18-kira-012-production-api-migration-return.md","docs/plans/zircon_plugins/02/2026-07-18-kira-graph-sync-repeated-compilation-return.md","docs/plans/zircon_plugins/02/2026-07-18-kira-send-target-bus-contract-return.md","docs/plans/zircon_plugins/02/2026-07-18-kira-test-module-topology-hardcut-return.md","docs/plans/zircon_plugins/02/2026-07-18-sound-automation-nonfinite-preflight-return.md","docs/plans/zircon_plugins/02/2026-07-18-sound-kira-root-lockfile-drift-return.md","docs/plans/zircon_plugins/02/failure-2026-07-17-kira-012-production-api-migration.md","docs/plans/zircon_plugins/02/failure-2026-07-17-kira-graph-sync-repeated-compilation.md","docs/plans/zircon_plugins/02/failure-2026-07-17-kira-send-target-bus-contract.md","docs/plans/zircon_plugins/02/failure-2026-07-17-kira-test-module-topology-hardcut.md","docs/plans/zircon_plugins/02/failure-2026-07-17-sound-kira-root-lockfile-drift.md","docs/plans/zircon_runtime/render/01/fixed-2026-07-18-sound-automation-nonfinite-preflight.md","docs/zircon_plugins/sound/runtime.md","zircon_plugins/Cargo.lock","zircon_plugins/sound/runtime/src/automation/target/apply.rs","zircon_plugins/sound/runtime/src/automation/target/effect/apply.rs","zircon_plugins/sound/runtime/src/automation/target/effect/base_parameters.rs","zircon_plugins/sound/runtime/src/automation/target/effect/common.rs","zircon_plugins/sound/runtime/src/automation/target/effect/delay.rs","zircon_plugins/sound/runtime/src/automation/target/effect/dynamics.rs","zircon_plugins/sound/runtime/src/automation/target/effect/filter.rs","zircon_plugins/sound/runtime/src/automation/target/effect/gain.rs","zircon_plugins/sound/runtime/src/automation/target/effect/mod.rs","zircon_plugins/sound/runtime/src/automation/target/effect/modulation.rs","zircon_plugins/sound/runtime/src/automation/target/effect/reverb.rs","zircon_plugins/sound/runtime/src/automation/target/effect/shaper.rs","zircon_plugins/sound/runtime/src/automation/target/effect/stereo.rs","zircon_plugins/sound/runtime/src/automation/target/helpers.rs","zircon_plugins/sound/runtime/src/automation/target/listener.rs","zircon_plugins/sound/runtime/src/automation/target/mod.rs","zircon_plugins/sound/runtime/src/automation/target/parameter_values.rs","zircon_plugins/sound/runtime/src/automation/target/source.rs","zircon_plugins/sound/runtime/src/automation/target/track.rs","zircon_plugins/sound/runtime/src/automation/target/volume.rs","zircon_plugins/sound/runtime/src/descriptor_validation/common.rs","zircon_plugins/sound/runtime/src/descriptor_validation/coordinates.rs","zircon_plugins/sound/runtime/src/descriptor_validation/listener.rs","zircon_plugins/sound/runtime/src/descriptor_validation/mod.rs","zircon_plugins/sound/runtime/src/descriptor_validation/source/tracks.rs","zircon_plugins/sound/runtime/src/descriptor_validation/volume.rs","zircon_plugins/sound/runtime/src/engine/state/graph.rs","zircon_plugins/sound/runtime/src/kira_bridge/manager.rs","zircon_plugins/sound/runtime/src/kira_bridge/manager/graph.rs","zircon_plugins/sound/runtime/src/kira_bridge/manager/lifecycle.rs","zircon_plugins/sound/runtime/src/kira_bridge/playback_data.rs","zircon_plugins/sound/runtime/src/service_types/mixer_graph/configuration.rs","zircon_plugins/sound/runtime/src/service_types/mixer_graph/sync.rs","zircon_plugins/sound/runtime/src/service_types/mod.rs","zircon_plugins/sound/runtime/src/tests.rs","zircon_plugins/sound/runtime/src/tests/automation_binding/validation.rs","zircon_plugins/sound/runtime/src/tests/automation_binding/validation/non_finite_values.rs","zircon_plugins/sound/runtime/src/tests/automation_binding/validation/unsupported_parameter/track_delay.rs","zircon_plugins/sound/runtime/src/tests/common.rs","zircon_plugins/sound/runtime/src/tests/common/assertions.rs","zircon_plugins/sound/runtime/src/tests/common/assets.rs","zircon_plugins/sound/runtime/src/tests/common/assets/clip.rs","zircon_plugins/sound/runtime/src/tests/common/assets/clip/builders.rs","zircon_plugins/sound/runtime/src/tests/common/assets/clip/validation.rs","zircon_plugins/sound/runtime/src/tests/common/effects.rs","zircon_plugins/sound/runtime/src/tests/common/listener.rs","zircon_plugins/sound/runtime/src/tests/kira_bridge/graph/routing.rs","zircon_plugins/sound/runtime/src/tests/kira_bridge/lifecycle/backend.rs","zircon_plugins/sound/runtime/src/tests/kira_bridge/source/runtime.rs","zircon_plugins/sound/runtime/src/tests/kira_graph_sync.rs","zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity.rs","zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity/helpers.rs","zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity/linked_manifests.rs","zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity/registration_reports.rs","zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity/runtime_modules.rs","zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity/static_manifest.rs","zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity/support.rs","zircon_plugins/sound/runtime/src/tests/runtime_core/registration/options/channel_layout.rs","zircon_plugins/sound/runtime/src/tests/support.rs","zircon_plugins/sound/runtime/src/tests/support/assertions.rs","zircon_plugins/sound/runtime/src/tests/support/assets.rs","zircon_plugins/sound/runtime/src/tests/support/assets/clip.rs","zircon_plugins/sound/runtime/src/tests/support/assets/clip/builders.rs","zircon_plugins/sound/runtime/src/tests/support/assets/clip/validation.rs","zircon_plugins/sound/runtime/src/tests/support/effects.rs","zircon_plugins/sound/runtime/src/tests/support/listener.rs"]

# Plugins02 M1：Kira hard cut 当前源码闭环

## 状态

`ready_for_milestone_commit / validation_review_green`

## Scope Delivered

- `zircon_plugin_sound_runtime` 已将执行核心硬切到 Kira 0.12.2；canonical root/plugin lockfile 分别固定当前依赖闭包，旧自研软件 mixer、CPAL output lifecycle 与对应兼容入口不再作为产品执行路径。
- Mixer Graph 编译、事务同步、播放控制、递归 post-effect send、bus/master gain 与 active graph resync 已落入 Kira graph owner；send 路由先编译目标 track，再按 parent/bus/master 语义组合增益，避免重复应用 master gain。
- Kira 0.12 production API 迁移及旧 test module graph 已完成：设备选择、播放速率、output lifecycle、`tap_mix`/`sidechain`/catalog support 旧挂载不再阻断 lib-test 编译。
- 生产 poison-lock 路径与 R2.3 `common/helpers` 无主模块债已清零；共享断言、资产、effect/listener fixture 和 optional-feature parity support 已迁到 named owner。
- 自动化参数有限值边界完成 RED→GREEN：Track、Volume、Effect 三条直接入口用 `NaN` 证明统一入口在资源查找与任何图/描述符副作用前返回 typed `SoundError::InvalidParameter`；共享 preflight 已落在 `apply_automation_target` 的 active-state gate 之后、target 分派之前。
- Mixer Graph mutation 现在以 revision + Kira active-state 双 CAS 分隔 inactive neutral authoring 与 active M1 编译；MockBackend active public-commit harness 复用 production 锁提交原语，覆盖 10/100/1000 track 的 add/update/remove/send 锁窗口和线性预算。
- Kira backend 的 logical limit 与 physical sub/send capacity 分开 preflight；exact-capacity 成功、capacity+1 typed failure 与 installed graph 不变由独立边界测试约束。

## Fresh Testing Evidence

- route RED：job `99687c8d6c584399aa727b09e121cdc1` / run `dc0d773d6c1e4b8b9b2b610114f7867c`，`4 executed`，`1 passed / 3 failed / 325 filtered`。
- current-source route GREEN：job `7016d604dcf84f75bb0ceac48b331660` / run `14646d4b335e4e19a2e20ad430cfd00f`，`8 passed / 0 failed / 334 filtered`，退出码 `0`；原三条 send/gain/sync 失败和新增 chained-send/active-sync 行为均已通过。
- automation RED：job `770baa08b4f24797b1ca0db228e4b2c9` / run `b7448e7c3d0f41ea92b012ed467a00b8`，`0 passed / 3 failed / 339 filtered`；focused GREEN：job `da450ccadd43494991f956627704041f` / run `e9a73126013d4c3e92ee979a485b0ef0`，`3 passed / 0 failed / 339 filtered`。
- plugin broad RED→收敛证据：job `c9675fc551c14a59a653eae57d539ca7` / run `6f38ded77b564959a13fa4640d01f7a6`，`341 passed / 1 failed / 0 filtered`；唯一失败是旧常量 96 allocation ceiling，随后改为 bounded scale budget，并按独立评审补入 active public CAS/lock harness。
- final current-source focused GREEN：job `402c6c99e45d45489082cdffa3154d05` / run `d970eae11471443db38ac234ccd53114`，`1 passed / 0 failed / 343 filtered`，退出码 `0`。active public lock p95 在 10/100/1000 tracks 的逐规模最坏值为 `224us / 1.349ms / 6.166ms`，低于 `7.5ms / 30ms / 255ms` 线性预算；1000-track active Kira mutation 最坏 p95 为 `75.562ms`，低于 `250ms`，allocation p95 为 `3114`，低于 `4*n+256`。
- final current-source plugin broad GREEN：job `93780d78e3184784b545160381387ff7` / run `b3ffbac30f36487ca6d1022fe916b9e1`，`344 passed / 0 failed / 0 filtered`，doc-tests `0 passed / 0 failed`，退出码 `0`；poison recovery 用例的显式 panic 为测试预期且最终结果为 `ok`。
- canonical package check GREEN：job `473424cc901640d18385d5767e257fbe` / run `beb24285ac9649b3a8f7060b3624b000`，`cargo +1.94.1 check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked --jobs 1` 退出码 `0`，当前源码完成 dev profile 编译。
- plugin workspace locked metadata GREEN：job `2eae0c7c12ad4801aa3d1ced603ca458` / run `d0d76d4d447c4261b8d7cca68825a719`，`cargo +1.94.1 metadata --manifest-path zircon_plugins/Cargo.toml --locked --format-version 1` 退出码 `0`。
- root workspace locked metadata GREEN：job `5bbd0ad81d9c41e7b299f611cab775a2` / run `b571b6ab82b841a5ae56853af6cdacb4`，`cargo +1.94.1 metadata --locked --format-version 1` 退出码 `0`。
- failure lifecycle：Sound plan 的 open failure 数为 `0`；Kira 0.12 API、repeated graph compilation、send/bus contract、test topology、canonical lock drift 与 automation non-finite preflight 共 6 条 child-record-only 交接均已由 coordinator 原子回传为 fixed，并生成 origin fixed record 与 Sound return receipt。
- failure audit：本次 6 条 Sound fixed node 均通过导入；全仓审计仍报告 Editor16、EditorUI06、Frameworks01/04、Render04、Runtime12 等外部计划的既有 schema diagnostics，不属于 Sound M1 manifest，也未由本会话吸收。
- canonical lock SHA-256：root `309BF641F1BF22D7E7BD4F4C4E7476325DC9F1754F8CE605135B4C1DAA811645`；plugin `181BDC2DDC3F394461A8D0D3230F10519E9D0631A76F59AE8F0E176A2D83F16B`。
- Sound structure audit：M1 classified-and-clear；manifest schema、shim、capability、dist boundary、skeleton debt 均为 `0`；当前生产文件最大 547 行，测试文件最大 526 行，无 >=700 行文件。
- 静态检查：生产 `lock().unwrap/expect`、`common/helpers`、`todo!/unimplemented!` 均为 `0`；scoped `git diff --check` 通过，仅仓库既有 LF/CRLF 提示。
- 独立只读评审首轮为 Critical `0` / Important `3`；契约/状态记录两项和 active public lock benchmark 一项均已按建议修改。第二轮复审为 Critical `0` / Important `0` / Minor `1`，verdict `READY_FOR_FRESH_VALIDATION`；Minor 是后续可补的 active-state 强制 race hook，不阻断当前 validation，最终证据写入后仍须登记 milestone review。
- 最终独立评审先发现 origin test-topology fixed record 继承了 4 个已删除 `related_code` 路径（Critical `0` / Important `1`），已改指真实存在的 Kira routing/graph-sync 与 output-device validation owners；快速复核为 Critical `0` / Important `0` / Minor `1`，verdict `READY`。唯一 Minor 仍是后续 active-state 强制 race hook，不阻断 M1。
- coordinator 提交前精确脏清单为 `86` paths；受保护的 `02-sound.md` 与 Runtime options 均 clean，foreign root `Cargo.lock` 明确排除，shared staged count 为 `0`。

## Review

- 独立终审 Critical `0` / Important `0` / Minor `1`，verdict `READY`；唯一 Minor 为后续 active-state 强制 race hook，不阻断 M1。
- 终审发现的 origin test-topology fixed record 旧 `related_code` Important 已改指 4 个真实存在的 current owners，并由同一 reviewer 快速复核归零。

## 待完成验收

active public graph benchmark focused、完整 plugin broad、package check 与双 workspace `cargo metadata --locked` 已在同一 1447-file current-source manifest（fingerprint `b2b7ce31ff8866bfce4407633e69292810653aab22cf5b24bed2e596afc5b3e3`）上 GREEN，6 条 child-record-only failure return 与最终独立复审 Critical/Important `0/0` 已完成。只剩执行 M1 coordinator milestone commit 并取得 immutable SHA；在 SHA 产生前仍不向 Render01 F2 / Shader06 放行。
