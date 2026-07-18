---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: rhi-wgpu-ui-surface-borrowed-cache-key-compile-blocker
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/zircon_runtime/runtime/12
fixing_child_dir: docs/plans/zircon_runtime/render/17
origin_workflow_node: M4
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
tests:
  - cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1
resolved_at: 2026-07-19
---


# Render17：RHI WGPU UI surface借用缓存键编译阻断

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
- 来源执行切片：M4 current-source canonical `zircon_runtime` check
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：最低共享原因位于Render17拥有的WGPU UI surface图片缓存查找；Runtime12不能修改该owner路径或用局部绕过伪造canonical check通过。

## 失败现象与复现证据

Windows受管job `693431b2f19c45cf9d9e7d98b1032568` / run `de368d8c08c24c7780e2c76b6d92c577` 执行 `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1`，source-bound终态为exit 101。唯一编译错误位于`zircon_runtime/src/rhi_wgpu/ui_surface.rs:301`：`HashMap<String, _>::get`接收到`&cache_key`，而`cache_key`已经是`&str`，因此类型推导要求未实现的`String: Borrow<&str>`（E0277）。该job未完成Runtime12 canonical gate，不能声称通过。

## 最低共享层根因

图片资源键从owned `String`硬切换为借用`&str`时，其余`insert`与`get_mut`调用已使用真实`str`借用合同，但replace判定残留了迁移前的额外取引用。正确查找键是`cache_key: &str`本身，不需要兼容层、克隆或调用点特例。

## 架构修复验收

- 将残留调用硬切换为`image_cache.get(cache_key)`，保持`HashMap<String, _>`唯一缓存owner及零额外键克隆。
- `rustfmt +1.94.1 --edition 2024 --config skip_children=true --check zircon_runtime/src/rhi_wgpu/ui_surface.rs`与精确`git diff --check`通过。
- 用包含当前全部dirty runtime/interface/Cargo/Runtime12清单的Windows受管同形`cargo check`获得source-bound exit 0，并确认无存活进程。

## 禁止临时方案

- 不得恢复每present键克隆、添加alias/兼容shim、改缓存键类型或绕过replace判定。
- 不得缩窄Runtime12 canonical source manifest、cfg-gate该renderer、降低测试或计划验收标准。

## 修复结果与回传

- 根因：The ui-surface cache lookup already held cache_key as &str, but the migrated call passed &cache_key, requiring HashMap<String, _> to satisfy String: Borrow<&str> and causing E0277.
- 架构修复：Hard-cut the real lookup to image_cache.get(cache_key), preserving String cache ownership, borrowed str lookup, zero additional clone, and no compatibility shim.
- 验证：Owner-only rustfmt check and git diff --check passed. Fresh Runtime12 source-bound cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1 job f6841642e70c4a43b8674c92f9f18461 run 230eaecd12ce4bfe97d92753efff6cdc reservation 6dd22367dd5041b88ad27c024ceb07ec used a 630-path manifest fingerprint 42216a3cdd88bbed30369bcce67ad5698effc2dfccfca6f08722e22ce27b1e44 binding ui_surface SHA-256 D1FAA0C11732CC948ED797AFC88154BCA2BC18092260EF06CF0FBE402925D9F0, finished in 21.53s, released exit 0, live_process_pids=[], and post-run audit matched 630/630 hashes with zero new relevant dirty paths outside the manifest.
- 回传：Runtime12 M4 canonical compilation may resume past the Render17 ui-surface borrowed-key blocker. The separate Render17 pairwise-overlap batching failure remains open and is not absorbed by this return.
