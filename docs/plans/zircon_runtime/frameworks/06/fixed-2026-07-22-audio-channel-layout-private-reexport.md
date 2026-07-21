---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: audio-channel-layout-private-reexport
origin_plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
fixing_plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/06
fixing_child_dir: docs/plans/zircon_runtime/frameworks/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/audio/channel_layout.rs
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/graph.rs
  - zircon_runtime/src/core/framework/sound/mix.rs
  - zircon_runtime/src/core/framework/sound/options.rs
  - zircon_runtime/src/core/framework/sound/output.rs
  - zircon_runtime/src/core/framework/sound/playback.rs
  - zircon_runtime/src/core/framework/sound/status.rs
  - zircon_runtime/src/core/framework/sound/tests.rs
  - tools/tests/test_frameworks_03_audio_contract_owner_boundary.py
  - docs/zircon_runtime/core/framework/audio.md
tests:
  - python -m unittest tools.tests.test_frameworks_03_audio_contract_owner_boundary -v
  - python tools/check_conventions.py --only docs --json
  - cargo check -p zircon_runtime --lib --no-default-features --features sound-contracts --locked
resolved_at: 2026-07-22
---


# Frameworks03: Audio channel layout 私有重导出残留

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`
- 来源执行切片：Frameworks06 G7 Batch13 Audio 中立契约文档 owner 硬切
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md`
- 交接原因：最低共享原因是 `sound-contracts` 内部仍通过 Sound root 重导出中立 Audio DTO，属于 Frameworks03 的可选契约边界，而不是 G7 路径解析器。

## 失败现象与复现证据

`docs/zircon_runtime/core/framework/audio.md` 声明 Sound namespace 不重导出 `AudioChannelLayout`，但 `zircon_runtime/src/core/framework/sound/mod.rs` 仍有 `pub(crate) use crate::core::framework::audio::AudioChannelLayout;`。七个 Sound leaf owner 与测试通过 `super::AudioChannelLayout` 间接消费，因此文档 focused G7 虽为 0，独立复审仍为 Important 1。

## 最低共享层根因

Audio DTO 已硬切到唯一声明 owner `core/framework/audio/channel_layout.rs`，但 Sound leaf imports 未在同批改成直接依赖中立 owner，遗留 crate-visible namespace alias。该 alias 扩大 Sound root surface，并让文档所有权与源码事实分叉。

## 架构修复验收

- 删除 Sound root 的 `AudioChannelLayout`/`AudioSpeakerChannel` 重导出；所有 leaf 与测试直接 import `core::framework::audio`。
- focused owner-boundary guard RED→GREEN，并证明 Sound root/leaf 内旧间接 import 为 0、旧 `sound/channel_layout.rs` 仍不存在。
- Audio 文档 front matter focused G7 为 0，正文“不从旧 Sound namespace 重导出”与源码一致。
- 在 coordinator-owned immutable validation copy 可用后，受管 `sound-contracts` Runtime 编译门通过。

## 禁止临时方案

- 不得把文档降格为“只保证无 public re-export”来掩盖 crate-visible alias。
- 不得把 alias 改名、移动到另一 façade、增加 shim 或让 leaf 继续从 `super` 间接导入。
- 不得把现有 Frameworks03 大守卫文件的 foreign 修改覆盖或回滚。

## 修复结果与回传

- 根因：Sound root retained crate-visible neutral Audio DTO reexports after the canonical owner moved to core/framework/audio, so seven leaves and tests still consumed the obsolete facade; malformed discrete_* names could also degrade into custom layouts.
- 架构修复：Removed both Sound root reexports, changed every leaf and contract test to import core::framework::audio directly, hardened named/discrete canonical validation and speaker uniqueness, and added mutation-resistant owner-boundary guards without aliases or shims.
- 验证：Owner-boundary unittest 6/6; Rust 1.94.1 rustfmt and diff-check GREEN; owned docs G7 0; managed job 46d9bf94358d4609b9b3866b65ecf71b run 9cd06735d00c47e1bb13900b258e3486 finished sound-contracts lib check in 5m05s with exit 0 and only 75 existing warnings; pre/post exact13 fingerprint a08af2af8713d674e6bf3c43d3b8bf74f773b045e7096977733728785d045d9c.
- 回传：Returned the fixed Audio/Sound owner hard cut to Frameworks06 Batch13; Frameworks03 and Frameworks06 continue their remaining milestones.
