---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: random-checkpoint-authority-generation-mismatch
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/22
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_contracts/src/random/stream_checkpoint.rs
  - zircon_runtime/crates/zr_contracts/src/random/service_checkpoint.rs
  - zircon_runtime/crates/zr_contracts/src/random/checkpoint_error.rs
  - zircon_runtime/crates/zr_contracts/src/random/tests/checkpoint.rs
  - zircon_runtime/src/core/runtime/random/registry.rs
  - zircon_runtime/src/core/runtime/random/service.rs
  - zircon_runtime/src/core/runtime/random/tests/service.rs
  - zircon_runtime/src/core/runtime/random/tests/retention.rs
tests:
  - cargo test -p zr_contracts random::tests::checkpoint
  - cargo test -p zircon_runtime core::runtime::random::tests
---

# Runtime22: random checkpoint authority generation mismatch

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Frameworks01 random checkpoint contract hardening review
- 修复责任计划：`docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md`
- 交接原因：最低共享根因位于 Runtime22 拥有的 random checkpoint capture、restore 与 eviction 生命周期。

## 失败现象与复现证据

`RandomStreamCheckpoint` 只持有 `key + RandomState`，而
`RandomServiceCheckpoint::validate` 只校验格式、算法和 key 顺序。因此外部输入可将 generation
较新的 `RandomServiceState` 与旧 generation 的 stream progress 拼接为一个可反序列化 checkpoint。
恢复后，已注册 key 延续旧状态，未注册 key 则按新 seed/generation 派生；同一服务包含两个从未原子共存的
authority era。world/entity eviction 返回的独立 stream checkpoint 也缺少 generation，无法证明来源时代。

## 最低共享层根因

持久化 stream contract 没有绑定 `master_seed_generation`，service contract 因而没有足够信息拒绝跨时代
组合。先前 `registry -> seed` 原子 capture 修复只封闭了内部调度交错，无法验证构造器或 serde 输入。

## 架构修复验收

- `RandomStreamCheckpoint` 强制携带 authority generation，且 service checkpoint 拒绝任一 stream generation
  与 service generation 不一致。
- checkpoint 格式硬切到 v2；v1 缺字段或旧版本输入必须失败，不保留兼容反序列化、别名或 fallback。
- 完整 checkpoint 与 world/entity eviction 均在唯一 `registry -> seed` 锁序内捕获 generation。
- restore/replay 对已注册及新 key 保持同一 authority era，并复现已注册 key 的 exact next draw/index。
- `evict_stream` 的裸 `RandomState` 仅表示移除状态，不得被描述为可独立恢复 checkpoint。

## 禁止临时方案

- 不添加 v1 兼容字段默认值、serde fallback、旁路构造器、重复 generation 真相或调用点特判。
- 不在释放 registry lock 后读取 generation，也不引入 `seed -> registry` 的第二锁序。
- 不削弱 malformed checkpoint、restore/replay 或 eviction 回归来隐藏失败。

## 修复结果与回传

实现进行中；当前不声明测试、性能、提交、推送或回传通过。

当前修复侧哈希（2026-08-31）已记录如下：

- `stream_checkpoint.rs`: `a353e505a5b674c81f864ab13130acd10cd4f18e37f042048c592e06c1bde44c`
- `service_checkpoint.rs`: `34a8c276f4e2c969e2069d30dfd368d31009f7ee7b91ce09055f0c4fd0edd739`
- `registry.rs`: `e40c79845fdc51c8a6e3177bd931d8b8166a55fb74c58a7a2aed61d90a955351`
- `service.rs`: `fe073099cf410e10765d848a1c8899dc11aca5affb09f9d1a9964ddc249681af`
- `tests/service.rs`: `3eeb0cdf0d28025cf48a23795385766595222d76b441f5db93086a7545e8de1c`
- `tests/retention.rs`: `e5010dac741f105c61a509fb802345330da18f81651a6d6a85de6634e37e068b`
- `registry/evict_matching_tests.rs`: `15b5cf62b28a0d984f39ea3b78a4e64388bd65d48402118d82376f0c2f83c0d5`

依赖快照哈希：`authority.rs`=`cdb5f29efcec25a84b6f2995fc4cd514f347a2dcc96fe27e799020dc7e4927e0`、
`service_state.rs`=`c10422917a2d8d1896c428c75173d6a7e9821adac2fdadb2af2f75aa93203f21`、
`service_checkpoint.rs`（本修复侧）=`34a8c276f4e2c969e2069d30dfd368d31009f7ee7b91ce09055f0c4fd0edd739`、
`checkpoint_error.rs`（Frameworks01 owner）=`a8507f27ebdc23f5a7075697b490c257fee39f50e25aef6ed4ae848ff6b02843`。

Remaining legal dependency: `checkpoint_error.rs` is still held by
`frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825` (`active`, current hash
`a8507f27ebdc23f5a7075697b490c257fee39f50e25aef6ed4ae848ff6b02843`). Runtime22 must receive that
exact path through coordinator transfer before adding `StreamAuthorityGenerationMismatch` and
running managed compile/restore validation.
