# PFO-4d2b Shadow Slot Scene Workspace Plan

## 状态

- 日期：2026-08-27
- 当前状态：`source_implemented_static_checks_passed_dynamic_validation_pending`
- 范围：shadow atlas 每slot camera `SceneUniform`、buffer/bind-group生命周期及帧上传事务
- 前置：PFO-4d2a0/a1已收敛环境回退物理身份与shadow环境binding lease；动态WGPU验收仍pending。

## 结构重审

当前 `ShadowMapRenderer::record_atlas_commands_with_attachment_ops` 对每个活跃 `ShadowAtlasSlotPass`、每帧执行一次 `device.create_buffer_init` 和一次 `device.create_bind_group`。该路径位于graph录制阶段，slot数随directional cascades、spot/point lights增长，导致native resource churn为 `O(active slots * frames)`；它不是queue write微优化问题。

调用图确认 `ShadowFramePlan` 在graph执行前由outer compiled frame owner构建，且同一owner已经持有唯一 `frame_buffer_uploads`。graph成功后该batch才由backend受理并在所有graph command buffer前可见，因此无需在executor内加锁、扩散queue权限或改变scene layout动态offset ABI。

## Unreal对齐与设计

Unreal的shadow view/mesh pass会复用持久view uniform资源并通过render graph/RHI command owner更新内容，而不是在每个draw录制循环创建新的RHI buffer。Zircon采用等价边界：outer frame prepare、持久slot workspace、graph只读录制。

1. `ShadowMapRenderer`新增grow-only `ShadowSlotSceneWorkspace`，拥有一个大uniform buffer和每slot一个固定offset bind group。
2. stride按`min_uniform_buffer_offset_alignment`对齐，单binding range仍精确限制为`size_of::<SceneUniform>()`；不修改公共scene layout的`has_dynamic_offset=false`，避免所有scene pass ABI连锁变化。
3. required slot超过capacity时按next-power-of-two增长buffer并重建0..capacity bind groups；增长次数为`O(log max slots)`，旧WGPU handle由底层引用生命周期保活。零slot不创建资源。
4. 每帧只遍历active `atlas_passes`，把每个camera uniform写入对应stride位置的一个exact active-span payload；所有slot共享一个`Arc<[u8]>`和一个target buffer upload range。
5. outer compiled frame owner在graph前prepare并追加现有`FrameBufferUpload`。graph/materialization/backend admission失败时batch被丢弃；下一帧重新prepare，不需要易错的CPU committed shadow。
6. graph录制按slot ordinal借用prepared bind group；若workspace未准备则返回显式错误，不panic、不静默漏阴影。
7. 删除shadow recorder的queue参数。device仍用于pipeline admission和已有forward-shadow receiver bind group，不扩大resource owner权限。

## 量化与验收门槛

- 稳态每帧shadow slot native buffer create `N -> 0`、bind-group create `N -> 0`；buffer upload packet/range `N mapped initializations -> 1`。
- capacity增长时buffer create为1、bind-group create为new capacity；增长均摊，禁止按历史capacity每帧扫描或上传。
- CPU packing时间/内存为`O(active slots * aligned SceneUniform stride)`；零slot为0 allocation/0 upload。
- source门槛：record loop中`create_buffer_init`/`create_bind_group`为0、queue参数为0；prepare发生在shadow plan之后、graph执行之前，并在backend admission之前进入frame batch。
- 动态门槛：1/4/16/64 slot记录cold/growth/stable resource creates、upload bytes、CPU p50/p95/p99、GPU shadow pass、RenderDoc resource identity、PNG和功耗。未取得动态证据前不宣称瓶颈消失或性能达标。

## 当前完成项与静态结果

- `ShadowMapRenderer`新增grow-only `ShadowSlotSceneWorkspace`。capacity使用`checked_next_power_of_two`增长，一个uniform buffer按device `min_uniform_buffer_offset_alignment`分槽，每个bind group使用固定offset与精确`SceneUniform` binding size。
- outer compiled frame在`ShadowFramePlan`之后、graph执行之前调用`prepare_slot_scene_uploads`。所有active slot被打包进一个aligned payload和一个`WgpuBufferUpload` range，并追加既有`FrameBufferUpload`；零slot返回空batch。
- graph recorder按slot ordinal只读已准备bind group；workspace不足返回带prepared/requested数量的显式错误。recorder queue参数、per-slot `create_buffer_init`和per-slot bind-group create已删除。
- 静态计数：record queue参数0、buffer-init 0、bind-group create 0；prepare源码中capacity growth buffer create 1、bind-group helper 1、upload range 1。plan/prepare/graph/admission顺序检查通过。
- 三个触及Rust文件的局部`rustfmt --check`与scoped `git diff --check`通过。未运行Cargo或动态WGPU；1/4/16/64 slot create/reuse、PNG、RenderDoc、profile和功耗仍pending。
