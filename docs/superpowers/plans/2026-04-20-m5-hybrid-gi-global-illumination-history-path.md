---
related_code:
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_new/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/scene_frame_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_history/prepare_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/post_process/execute_post_process_stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_temporal_signature.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/record_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
implementation_files:
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_new/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/scene_frame_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_history/prepare_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/post_process/execute_post_process_stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_temporal_signature.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/record_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
plan_sources:
  - user: 2026-04-20 continue M5 Hybrid GI and make temporal/history data paths real under the current zircon_runtime graphics architecture
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - docs/superpowers/plans/2026-04-20-m5-hybrid-gi-final-resolve-runtime-parent-chain-gather.md
tests:
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_reuses_global_illumination_history_when_scene_history_resolve_is_disabled --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_rejects_global_illumination_history_when_probe_support_moves_off_pixel --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_preserves_more_history_when_hierarchy_resolve_weight_is_stronger --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_rejects_global_illumination_history_when_probe_identity_changes_at_same_support --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_rejects_global_illumination_history_when_deeper_probe_lineage_changes --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_ignores_non_dominant_probe_confidence_when_blending_history --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_blends_nonzero_exact_irradiance_entry_with_runtime_ancestor_gather --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_blends_nonzero_exact_rt_entry_with_runtime_ancestor_gather --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_blends_nonzero_exact_resolve_weight_with_runtime_ancestor_gather --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline requested_descendant_runtime --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline hybrid_gi_ --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline history_resolve_blends_previous_scene_color_when_enabled --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo check -p zircon_runtime --locked --offline --lib --target-dir D:/cargo-targets/zircon-workspace-compile-recovery
doc_type: milestone-detail
---

# M5 Hybrid GI Global-Illumination History Path

## Goal

把 `FrameHistorySlot::GlobalIllumination` 从“只在 feature descriptor 上声明”推进成真实可执行的数据路径：

- `Hybrid GI` 即使关闭通用 `HistoryResolve(SceneColor)`，仍然可以保有自己独立的 temporal history
- 历史数据不再混进整张 scene color，而是单独保存 post-process 输出的 GI indirect-light contribution
- 下一帧 `Hybrid GI` resolve 直接消费这份 GI history，而不是继续完全依赖 runtime host continuity 或 scene color history

这一步对应路线图里 `Hybrid GI -> screen-probe temporal accumulation / history reuse` 的第一条真实落地链路。

## Delivered Slice

### 1. Scene renderer 新增独立 GI history render target

`OffscreenTarget` 现在新增：

- `global_illumination`
- `global_illumination_view`

post-process pass 也从单目标输出改成了双目标输出：

- `final_color`
- `global_illumination`

其中第二个 attachment 专门记录当前帧 Hybrid GI resolve 的 indirect-light contribution，避免把 GI history 和整张 scene color 混成同一个 temporal source。

### 2. Frame history textures 真正承载 `GlobalIllumination` slot

`SceneFrameHistoryTextures` 现在新增：

- `global_illumination`
- `global_illumination_view`

而 `prepare_history_textures(...)` 也不再只在 `history_resolve` 或 `ssao` 开启时准备历史纹理。只要 `hybrid_global_illumination_enabled` 打开，GI history target 就会被创建并跨帧复用。

这意味着 `RenderFeatureDescriptor(global_illumination)` 声明的 `FrameHistorySlot::GlobalIllumination` 终于有了 renderer-side 对应资源，而不是停在 compile-time binding。

### 3. copy/execute 链路补齐 GI history 读写

`copy_history_textures(...)` 现在会在 `Hybrid GI` 打开时，把当前帧的 `target.global_illumination` 拷回 `history.global_illumination`。

`execute_post_process_stack(...)` / `execute_post_process(...)` 也已经把：

- previous GI history view
- current GI render target view

一起传入 post-process 执行链，所以 post-process shader 可以在 fragment resolve 时直接读取上一帧 GI contribution，再把当前帧 blended 结果写回新的 GI attachment。

### 4. post-process shader 新增独立 GI temporal blend

`post_process.wgsl` 现在新增：

- `history_global_illumination_tex`
- 双 attachment `FragmentOutput`
- `indirect_light` 的独立 history 混合
- support-aware 的 history rejection
- hierarchy-resolve-weight-aware 的 history confidence

行为上变成：

1. 先照旧算出当前帧 `indirect_light`
2. 如果 `hybrid_gi_counts.z != 0`，读取上一帧 `history_global_illumination_tex`
3. 按当前像素的 probe support 估算 `indirect_light_history_support`
4. 再按当前 probe 的 `hierarchy_resolve_weight` 推导 temporal confidence
5. 只有 support 足够时，才进入 history reuse；同时 hierarchy confidence 越强，允许保留的 history 越多
6. blended `indirect_light` 一份加到最终 `color`
7. 同一份 blended `indirect_light` 写入独立 GI history attachment

这样 `Hybrid GI` history 的 source 和 sink 都闭合到了同一条 render path 上，不再借道 scene color history。
同时：

- probe 已经移走、当前像素只剩弱尾巴 influence 时，旧 GI history 不会继续在屏幕上留下明显 ghost
- 同一屏幕 support 下，hierarchy resolve weight 更强的 lineage 会保留更多历史能量，而不是把 temporal accumulation 退化成完全 flat 的屏幕混合

### 5. GI history 现在带有 scene-driven probe temporal signature

这轮又把 `Hybrid GI` temporal validity 从单纯 screen-space support，推进到了 scene-driven lineage truth：

- `GpuHybridGiProbe` 新增 `temporal_signature_and_padding`
- `encode_hybrid_gi_probes(...)` 新增 `hybrid_gi_temporal_signature.rs`
- temporal signature 不再只混 `probe_id + immediate parent_probe_id`
- helper 会继续沿 `RenderHybridGiExtract` 的 `parent_probe_id` chain 向上 fold 完整 ancestor lineage，再量化成 q8 signature
- post-process shader 把 dominant probe 的 temporal signature 写进独立 GI history attachment 的 alpha
- 下一帧读取 `history_global_illumination_tex` 时，会先比较 current dominant signature 与 history signature；不匹配就把这次 GI history blend 直接压成 0

因此 temporal reuse 现在已经不只是“像素 support 还在不在”，而是开始明确绑定：

- dominant probe identity 是否还相同
- 这个 probe 背后的 scene-driven hierarchy / ancestor lineage 是否还是同一条 truth

这让 `Hybrid GI` 的 temporal accumulation 更接近 Lumen-style probe lineage continuity，而不是继续把历史有效性停在 purely screen-space ghost rejection。

### 6. Temporal confidence 现在只跟 dominant probe 走，不再被无关 probe 污染

在这轮收口前，`post_process.wgsl` 对 `indirect_light_history_confidence` 取的是所有 resident probe 的 `max(...)`：

- 即使某个高 `hierarchy_resolve_weight` probe 根本不覆盖当前像素
- 它也会抬高当前像素的 GI history blend
- 导致非 dominant、无 screen support 的 lineage 把不属于它的 temporal retention 一并放大

现在 shader 已经改成：

- 先用 `probe_history_support` 决定当前像素的 dominant probe
- 只把这个 dominant probe 的 `hierarchy_resolve_weight -> temporal confidence` 绑定到 history blend
- 非 dominant probe 即使 hierarchy weight 更高，只要对当前像素没有更强 support，就不会再污染这次 temporal reuse

这一步把 temporal confidence 进一步收敛到了“当前像素真正消费的 scene-driven probe truth”上，而不是继续停在全局 probe-max heuristic。

### 7. Final resolve 不再让 non-zero exact runtime entry 硬遮蔽 ancestor continuation

另一条刚收掉的缺口在 `encode_hybrid_gi_probes/*` 的 runtime hierarchy consume 语义上：

- 之前只要 exact probe runtime entry 存在且 `weight > 0`
- `hybrid_gi_hierarchy_resolve_weight(...)`、`hybrid_gi_hierarchy_irradiance(...)` 与 `hybrid_gi_hierarchy_rt_lighting(...)` 都会直接短路
- 更上层 ancestor runtime continuation 即使更强，也完全进不到 final resolve

现在这条 exact-or-parent 短路已经被拆掉：

- `runtime_parent_chain.rs` 新增了共享的 `blend_runtime_rgb_sources(...)`
- `hybrid_gi_hierarchy_resolve_weight.rs` 也新增了独立的 resolve-weight blend support 映射
- exact runtime source 先保留
- scene-driven parent-chain gathered runtime source 继续收集
- resolve weight / irradiance / RT-lighting 三条 source 都开始按各自 `weight/support` 做加权混合
- 只有真正没有 exact 或 inherited source 时才回退到旧路径

这样 `Hybrid GI` final resolve 已经从“exact 或 parent 二选一”推进到“exact + parent-chain continuation 共同生效”。这意味着：

- non-zero 但较弱的 exact leaf resolve-weight 不会再把更强的 ancestor resolve continuation 全吃掉
- non-zero 但较弱的 exact leaf entry 不会再把更强的 ancestor irradiance continuation 全吃掉
- non-zero 但较弱的 exact RT entry 也不会再把 ancestor RT-lighting continuation 全吃掉
- final resolve 开始更像 scene-driven screen-probe hierarchy gather，而不是 leaf-local short-circuit

### 8. Final resolve 现在会在 parent exact entry 缺失时继续 gather requested descendant runtime continuation

这轮收掉的下一条缺口在 current extract 的 child-chain continuation：

- 之前 `hybrid_gi_hierarchy_resolve_weight(...)`
- `hybrid_gi_hierarchy_irradiance(...)`
- `hybrid_gi_hierarchy_rt_lighting(...)`

都只会消费：

- exact probe runtime entry
- current extract parent-chain gathered runtime ancestor continuation

因此一旦 resident parent probe 自己的 exact runtime hierarchy entry 缺失，即使 runtime host 里仍然保有 request-driven child probe 的 hierarchy resolve / irradiance / RT-lighting truth，final resolve 也会直接掉回 flat/no-hierarchy 输出。

现在这条 child-chain continuation 已经补齐：

- `runtime_parent_chain.rs` 新增 `gather_runtime_descendant_chain_rgb(...)`
- `runtime_parent_chain.rs` 新增 `gather_runtime_descendant_chain_weight(...)`
- helper 会沿当前 `RenderHybridGiExtract` 的 `parent_probe_id` 反向 child links 遍历 descendant chain
- descendant support 使用和 runtime-side requested descendant lineage 一致的 `0.84` falloff
- final resolve 只在 exact runtime entry 缺失或被过滤成 zero-support 时，才把 descendant runtime continuation 合到 inherited source 里

这意味着：

- resident parent probe 如果暂时没有 exact runtime hierarchy entry
- 但 request-driven child probe 仍然保有 runtime hierarchy irradiance / RT / resolve-weight truth
- final resolve 现在会继续沿当前 scene-driven hierarchy gather 这份 descendant continuation
- 而不是把 parent merge-back probe 直接压回 flat baseline

同时 exact-parent source 仍然保持优先，不会把已经存在的 exact runtime parent entry 再和它自身 child-derived continuation 重复双算一遍。

## Why This Matters

之前 `Hybrid GI` 的“跨帧连续性”主要来自 runtime host：

- `HybridGiRuntimeState`
- `build_resolve_runtime()`
- GPU completion readback continuation

这些路径更接近 screen-probe hierarchy / radiance cache host truth，但它们还不是 renderer 级别的真正 temporal resolve contract。

这轮补完后，renderer 自己也开始拥有一条明确的 GI history 路径：

- feature 声明 `GlobalIllumination` history slot
- renderer 分配独立 history texture
- shader 读上一帧 GI contribution
- shader 写下一帧 GI contribution

因此 `Hybrid GI` 的 temporal continuity 不再完全寄生在 runtime-side hierarchy continuation 上，而是开始具备了更接近 Lumen final-composite 的独立 temporal resolve 语义。

## Validation Summary

### Red/Green

- 红灯
  - `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_reuses_global_illumination_history_when_scene_history_resolve_is_disabled --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- 绿灯
  - 同上命令在 GI history path 接入后通过

### Regression

- `cargo test -p zircon_runtime --locked --offline hybrid_gi_ --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline history_resolve_blends_previous_scene_color_when_enabled --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo check -p zircon_runtime --locked --offline --lib --target-dir D:/cargo-targets/zircon-workspace-compile-recovery`

### Extra Temporal Rejection Coverage

- `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_rejects_global_illumination_history_when_probe_support_moves_off_pixel --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_rejects_global_illumination_history_when_probe_identity_changes_at_same_support --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_rejects_global_illumination_history_when_deeper_probe_lineage_changes --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_ignores_non_dominant_probe_confidence_when_blending_history --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`

### Exact + Ancestor Runtime Continuation Coverage

- `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_blends_nonzero_exact_resolve_weight_with_runtime_ancestor_gather --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_blends_nonzero_exact_irradiance_entry_with_runtime_ancestor_gather --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_blends_nonzero_exact_rt_entry_with_runtime_ancestor_gather --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`

### Requested Descendant Continuation Coverage

- `cargo test -p zircon_runtime --locked --offline requested_descendant_runtime --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline hybrid_gi_ --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`

### Hierarchy-Weighted Temporal Coverage

- `cargo test -p zircon_runtime --locked --offline hybrid_gi_resolve_preserves_more_history_when_hierarchy_resolve_weight_is_stronger --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture`

## Remaining Route

这条 temporal/history 链现在只是第一阶段闭环，后面仍有两条自然延伸：

- 把 current fragment-space GI history blend 继续推进到更完整的 screen-probe temporal accumulation policy
  - history rejection / confidence
  - per-probe temporal weighting instead of fixed screen-space blend
- 把 runtime host 的 hierarchy-aware resolve truth 与 renderer temporal truth 进一步收敛
  - scene-driven screen-probe hierarchy gather
  - probe gather/request continuation
  - RT hybrid-lighting continuation 和 temporal accumulation 的更深融合
