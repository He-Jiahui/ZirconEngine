# UI 合批与交互完整校验计划

## Summary
- 先把 `drawer_resize` 的真实拖拽链路校验清楚：现有 profile 脚本用固定比例点起拖，可能没有命中实际 splitter；验收前必须改成按 live host geometry 定位分隔条。
- 校验范围限定在 `zircon_editor` retained host command stream 和 `zircon_runtime` WGPU UI surface；不改 `zircon_hub`/Slint。
- 最终输出一份完整结论：drag/resize/asset_refresh 严格 profile、关键页面 live GPU vs fallback 截图、点击命中区与渲染区一致性、UI 合批效率与偏序集合理性。

## Key Changes
- 增强 profile 捕获工具：
  - 为 profiling 模式导出 `ui_profile_geometry.json`：窗口 client size、resize splitters、document/drawer tab frames、viewport frame、主要 clickable frames、当前 presenter backend。
  - `drawer_resize` 改为使用 `resize_layer.left/right/bottom_splitter_frame` 的中心点拖拽，不再用固定比例。
  - `drag` 改为从实际 document/drawer tab frame 起拖，并要求产生 `ui.drag` redraw + GPU batch evidence。
  - `asset_refresh` 使用临时 profile project，自动触发 harmless asset file touch/change，要求产生 `ui.asset_refresh` evidence。
- 增加 profile 指标报告：
  - `batch_success_rate = 1 - gpu_draw_calls / gpu_visible_draw_items`
  - `draw_reduction_ratio = gpu_visible_draw_items / gpu_draw_calls`
  - `dependency_density = gpu_batch_dependencies / (n * (n - 1) / 2)`
  - `layer_density = gpu_visible_draw_items / gpu_batch_layers`
  - 报告 worst case：所有 item 重叠或材质无法合并时退化到接近逐 item draw。
  - 报告 ideal case：同层同 solid/text/同 resource image 分别合为 1 draw；混合层最小值为 `solid(0/1) + text(0/1) + image_resource_key_count`。
- 补齐视觉和命中校验：
  - 增加 live client screenshot capture：GPU 默认路径、profiling-only forced softbuffer fallback 路径、software reference snapshot 三份输出，并生成 diff summary。
  - 对 resize splitter、tabs、activity rail、viewport toolbar、template/list controls 做中心点和边界点采样，验证 hit route 与渲染 frame 一致；框外 1-2px 负采样不能命中同一控件。
- Mask/clip 结论：
  - 当前 command stream 只有矩形 `Clip`/damage，不支持任意形状 mask/stencil。
  - 本轮验证矩形 clip：solid/image 顶点和 UV 在 CPU 裁剪后不泄漏，text 继续使用 glyphon bounds。
  - 若未来加入非矩形 mask，必须作为 batch key/stencil 层或显式逐 item fallback，不能隐式破坏偏序合批结论。

## Public/Artifact Changes
- 不改变 `ChromeCommandStream` 和 runtime public command DTO。
- 新增 profiling-only 环境/输出：
  - `ZIRCON_PROFILE_FORCE_SOFTBUFFER=1` 只用于 fallback screenshot 对比。
  - `ui_profile_geometry.json`、`ui_batch_metrics.json`、`screenshot_gpu.png`、`screenshot_softbuffer.png`、`screenshot_reference.png`、`screenshot_diff.json`。
- `tools/ui-profile-capture.ps1` 输出合批成功率、draw reduction、dependency density、fallback 状态和 scenario evidence gate。

## Test Plan
- Focused correctness gates：
  - `cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1`
  - `cargo test -p zircon_runtime --lib ui_hotspot --locked --jobs 1`
  - `cargo test -p zircon_editor --lib command_stream --locked --jobs 1`
  - `cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1`
  - `cargo test -p zircon_editor --lib retained_drawer_resize --locked --jobs 1`
  - `cargo test -p zircon_editor --lib retained_tab_drag --locked --jobs 1`
  - `cargo test -p zircon_editor --lib native_host_resize_splitter_forwards_move_and_release_after_capture --locked --jobs 1`
  - `cargo check -p zircon_app --features target-editor-host --locked`
- Profile gates after geometry-based interaction is wired:
  - `tools/ui-profile-capture.ps1 -Scenario drag -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 4 -SkipBuild`
  - `tools/ui-profile-capture.ps1 -Scenario drawer_resize -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 4 -SkipBuild`
  - `tools/ui-profile-capture.ps1 -Scenario asset_refresh -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 4 -SkipBuild`
  - Then rerun `startup,idle_hover,click,viewport_image` once to ensure no regression.
- Screenshot/live parity:
  - Capture startup/workbench, drawer resize after drag, viewport image, and asset refresh states.
  - Acceptance: no software fallback in GPU run, fallback run is intentional only, major UI geometry aligns, non-text solid/image areas match within small color tolerance, text differences are bounded to glyph/text boxes.
- Workspace evidence:
  - Run `cargo build --workspace --locked --verbose --jobs 1`.
  - Run `cargo test --workspace --locked --verbose --jobs 1`.
  - If full workspace still fails or times out, record exact crate/test/log and classify whether it is unrelated to UI batching; focused gates remain the UI-stage acceptance evidence only with that blocker stated explicitly.

## Assumptions
- 不提交、不归并 unrelated dirty files；只整理本轮相关 diff 和验证报告，除非后续明确要求 commit。
- 本轮目标是“像素正确 + 可观测 draw-call 下降 + 交互命中真实可用”，不引入 depth/stencil 或 image atlas。
- 对 live profile 的合批最低验收是 `gpu_draw_calls < gpu_visible_draw_items`、`software_fallback_present_count == 0`、`alerts == 0`；列表合批能力用 synthetic/list-heavy fixture 要求接近 ideal case。

## Progress 2026-05-16
- 已补强 `tools/ui-profile-capture.ps1` 的严格证据门槛：`drawer_resize` 使用 live `resize_splitters` 几何起拖并要求 `resize_changed_layout=true`；`drag` 使用 live tab frame；`asset_refresh` 触碰临时 profile project 的资产文件并要求 asset/editor/resource change counter；`ui_hit_consistency.json` 要求中心点/负采样与共享 route 一致；`screenshot_diff.json` 现在记录阈值并对直接 GPU-vs-softbuffer 的 `differing_sample_ratio` 与 `average_channel_delta` 做硬失败判断。
- 已补 runtime planner 合批指标单测覆盖：disjoint list rows 期望 6 个 visible item 合为 2 个 draw、2 层、3 条依赖；all-overlap worst case 期望 4 个 visible item 退化为 4 个 draw、4 层、6 条依赖。
- 静态检查已通过：PowerShell parser、`rustfmt --edition 2021 --check zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs`、相关 diff 的 `git diff --check`。
- 代码层检查结论：可视 splitter 渲染、native top-level route、resize pointer bridge、profile geometry export 都使用同一组 `resize_layer.left/right/bottom_splitter_frame`；因此点击/拖拽区域与实际渲染区域已经被同源几何约束，仍需 live profile 证明系统输入事件能让布局实际变化。
- 当前阻塞：机器上持续存在多组其他会话启动的 `cargo`/`rustc`/`link` 进程，且 `target/profiling/zircon_editor.exe` 与 `zircon_runtime.dll` 尚未产出。独立 `CARGO_HOME`/`CARGO_TARGET_DIR` 的 focused runtime 单测在依赖编译早期以进程码 `-1` 结束，日志只有下载/编译依赖行，没有 Rust 诊断。因此 drag/drawer_resize/asset_refresh 严格 profile、截图级 live parity、full workspace test 还不能作为通过证据。
- 待资源释放后优先顺序：先跑 `cargo test -p zircon_runtime --lib batch_plan_batches_disjoint_list_rows_by_depth_and_material --locked --jobs 1` 与 all-overlap 单测，再构建 profiling 目标，最后跑 `drawer_resize`、`drag`、`asset_refresh` 的 `-AutoInteract -RequireScenarioEvidence`，并至少用 `startup`/`viewport_image` 加 `-CaptureSoftbufferScreenshot` 补截图 parity。

## Progress 2026-05-16 Live Validation
- WSL stable target `target/wsl-ui-validation` passed the runtime planner evidence gates:
  - `cargo test -p zircon_runtime --lib batch_plan_batches_disjoint_list_rows_by_depth_and_material --locked --jobs 1` (`.codex/validate-wsl-runtime-batching-list-row-stable.log`): 6 visible list-row items reduce to 2 draw calls, 2 batch layers, and 3 dependencies.
  - `cargo test -p zircon_runtime --lib batch_plan_degenerates_to_one_draw_per_item_when_all_items_overlap --locked --jobs 1` (`.codex/validate-wsl-runtime-batching-worst-stable.log`): 4 fully overlapping items stay at 4 draw calls, 4 layers, and 6 dependencies.
- `cargo test -p zircon_editor --lib profile_geometry --locked --jobs 1` passed 4/4 after routing the unrelated render-graph post-process record write through `graph_execution.record` instead of re-borrowing `graph_execution_record`. The editor tests cover absolute splitter/tab export, clip omission, viewport toolbar clipping, and top-hit filtering.
- Windows profiling artifacts were rebuilt in `target/ui-live-validation/profiling`; `zircon_runtime.dll` and `zircon_editor.exe` were both available for live captures.
- `drag` strict live profile passed with `tools/ui-profile-capture.ps1 -Scenario drag -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 10 -SkipBuild` (`20260516-213412-drag`): 164 visible draw items reduced to 39 real GPU draw calls, `batch_success_rate=0.762`, `draw_reduction_ratio=4.205`, `dependency_density=0.098`, `layer_density=9.647`, `hit_consistency_samples=93 failed=0`, and `software_fallback_present_count=0`.
- `viewport_image` strict metrics profile passed without screenshot parity (`20260516-214006-viewport_image`): 21 visible draw items reduced to 16 real GPU draw calls, `batch_success_rate=0.238`, `draw_reduction_ratio=1.312`, `dependency_density=0.843`, `layer_density=1.400`, `hit_consistency_samples=93 failed=0`, and no software fallback.
- `drawer_resize` live profile proves the mouse can drag the border but does not yet pass the strict batch gate. `20260516-213637-drawer_resize` used live geometry target `resize.left_splitter`, moved the left drawer by 80px, refreshed geometry, and passed hit consistency (`87 failed=0`), but the interaction scenario recorded `gpu_draw_calls=0` and `gpu_visible_draw_items=0`, so it cannot be used as a draw-call reduction sample.
- `asset_refresh` live profile proves the temporary project asset touch is observed but does not yet pass the strict batch gate. `20260516-213543-asset_refresh` recorded `asset_refresh_change_count=5` and hit consistency (`93 failed=0`), but the scenario recorded `gpu_draw_calls=0` and `gpu_visible_draw_items=0`.
- Screenshot parity is not accepted. `20260516-213715-startup -CaptureSoftbufferScreenshot` failed direct GPU-vs-softbuffer with `differing_sample_ratio=1.0000`; the live GPU screenshot was almost pure white and the forced-softbuffer screenshot was only 16x16, so that startup parity sample is an invalid capture. `20260516-213846-viewport_image -CaptureSoftbufferScreenshot` produced normal 1280x720 screenshots but still failed direct GPU-vs-softbuffer with `differing_sample_ratio=0.9998` and `average_channel_delta=63.05`, so visual parity remains an open defect/evidence gap.
- Full workspace tests remain outside this stage's current evidence: earlier workspace runs were blocked by Windows debug timeout and unrelated dirty workspace failures. Current acceptance is focused runtime/editor tests plus live profile evidence, with the screenshot parity and drawer/asset batch-gate gaps explicitly left open.

## Progress 2026-05-17 Parity Follow-Up
- Retained-host redraw attribution was fixed so later frame-update requests own coalesced redraw scenarios. `redraw_merge_uses_latest_frame_update_scenario` failed before the fix with `Click` instead of `DrawerResize`, then passed after the change; `cargo test -p zircon_editor --lib redraw --locked --jobs 1 --message-format short` also passed.
- The screenshot parity root cause was the runtime WGPU UI surface color path. `20260517-182730-viewport_image` reproduced direct GPU-vs-softbuffer `differing_sample_ratio=0.6522`; the fix moved the retained UI target to `Rgba8Unorm`, prefers non-sRGB swapchain formats, clears new retained targets to opaque black, and prefers opaque swapchain alpha. `cargo test -p zircon_runtime --lib wgpu_ui_surface_uses_non_srgb_formats_for_byte_exact_editor_parity --locked --jobs 1 --message-format short` passed.
- Rebuilt live evidence shows the requested parity/interaction blockers closed: `20260517-190736-viewport_image` direct GPU-vs-softbuffer `differing_sample_ratio=0.0165`, `average_channel_delta=0.9022`, `21` visible draw items to `16` GPU draw calls, and `93 failed=0` hit samples; `20260517-190840-drawer_resize` moved the live left splitter `80px`, refreshed geometry, kept `87 failed=0` hit samples, and recorded `653` visible draw items to `124` GPU draw calls; `20260517-190851-asset_refresh` recorded `266` visible draw items to `42` GPU draw calls and `114 failed=0` hit samples.
- This follow-up does not claim full zero-alert UI hotspot acceptance. Latest profile summaries still include follow-up alerts such as `click/non_structural_interaction_rebuilt_presentation`, `drawer_resize/resize_triggered_slow_path_rebuild`, `idle_hover/region_request_repainted_full_frame`, and `startup/gpu_presenter_recorded_no_draw_calls`; broad workspace validation also remains deferred.
