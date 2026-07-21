---
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
implementation_files:
  - tools/tests/test_frameworks_03_audio_contract_owner_boundary.py
  - docs/zircon_runtime/core/framework/audio.md
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
tests:
  - python -m unittest tools.tests.test_frameworks_03_audio_contract_owner_boundary -v
  - python tools/check_conventions.py --only docs --json
  - rustfmt --edition 2021 --check on the owned Rust paths
  - managed cargo check -p zircon_runtime --lib --no-default-features --features sound-contracts --locked
doc_type: milestone-detail
---

# Frameworks03 Audio Channel Layout Owner Hard Cut

Plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
Milestone: M1
Status: completed
Date: 2026-07-22
Files: ["docs/plans/zircon_runtime/frameworks/03/2026-07-22-audio-channel-layout-owner-hardcut.md", "docs/plans/zircon_runtime/frameworks/03/2026-07-22-audio-channel-layout-private-reexport-return.md", "docs/plans/zircon_runtime/frameworks/06/fixed-2026-07-22-audio-channel-layout-private-reexport.md", "docs/zircon_runtime/core/framework/audio.md", "tools/tests/test_frameworks_03_audio_contract_owner_boundary.py", "zircon_runtime/src/core/framework/audio/channel_layout.rs", "zircon_runtime/src/core/framework/sound/components.rs", "zircon_runtime/src/core/framework/sound/graph.rs", "zircon_runtime/src/core/framework/sound/mix.rs", "zircon_runtime/src/core/framework/sound/mod.rs", "zircon_runtime/src/core/framework/sound/options.rs", "zircon_runtime/src/core/framework/sound/output.rs", "zircon_runtime/src/core/framework/sound/playback.rs", "zircon_runtime/src/core/framework/sound/status.rs", "zircon_runtime/src/core/framework/sound/tests.rs"]

## Scope Delivered

| 切片 | 状态 | 完成证据 |
|---|---|---|
| Audio DTO 唯一 owner | implemented | `AudioChannelLayout` 与 `AudioSpeakerChannel` 只由常驻 `core::framework::audio` 声明；Sound root 的 crate-visible 重导出已删除。 |
| Sound leaf 显式依赖 | implemented | 七个 Sound leaf 与契约测试直接 import 中立 Audio owner，旧 `super::AudioChannelLayout`/`AudioSpeakerChannel` 为 0。 |
| Layout contract 收敛 | implemented | named layout 通过 canonical contract 校验；speaker 唯一性使用固定 bitset；全部 `discrete_*` 名称保留给严格 canonical discrete schema。 |
| 旧架构清理 | implemented | 旧 Sound channel-layout 文件和 plugin channel-layout 模块均不存在，没有 alias、shim 或兼容重导出。 |
| 静态与受管门禁 | completed | Python owner-boundary 6/6、scoped rustfmt/diff-check 与本切片文档 G7 已 GREEN；coordinator 登记的 r3 validation copy 上 Rust 1.94.1 `sound-contracts` 编译 exit 0。 |

## Architecture Decision

声道布局是资产、运行时 Sound 合同和插件实现共同消费的中立 DTO，不属于 Sound façade。
因此所有 Sound leaf 必须直接依赖 `core::framework::audio`，Sound root 不得重新暴露该类型。
自定义布局可以声明非保留名称与显式 speaker topology；`discrete_*` 命名空间只接受
`discrete_<channel_count>` 且 speaker 列表为空的 canonical 表达，不允许错误拼写降级为自定义布局。

## Fresh Testing Evidence

- `python -m unittest tools.tests.test_frameworks_03_audio_contract_owner_boundary -v`：6/6 GREEN。
- scoped `rustfmt --edition 2021 --check` 与 `git diff --check`：GREEN。
- 全仓 docs G7 当前仍有 583 个 foreign missing-path 违规；本记录与
  `docs/zircon_runtime/core/framework/audio.md` 的 owned violation 为 0。
- 旧间接 import、旧文件、旧 plugin 模块和 Sound root 重导出扫描均为 0。
- managed job `46d9bf94358d4609b9b3866b65ecf71b` / run
  `9cd06735d00c47e1bb13900b258e3486`：5m05s、exit 0，仅 75 条既有 warning；启动前后
  exact13 fingerprint 均为 `a08af2af8713d674e6bf3c43d3b8bf74f773b045e7096977733728785d045d9c`。

## Review

当前生产实现与 2026-07-19 独立 `Critical 0 / Important 0 / Minor 0` 复审输入一致；唯一
源码差异是 `sound/tests.rs` 的 rustfmt 换行。2026-07-22 对 exact15 的重新审阅同样为
`Critical 0 / Important 0 / Minor 0`；最终 fixed-return manifest 的 review evidence 由
coordinator milestone gate 持有。

## Remaining Scope

本切片只关闭 Audio/Sound owner boundary failure，不宣称 Frameworks03 M1、M2 或整份计划完成。
后续工作继续执行父计划剩余 feature/profile 矩阵与 current-main CI 验收。
