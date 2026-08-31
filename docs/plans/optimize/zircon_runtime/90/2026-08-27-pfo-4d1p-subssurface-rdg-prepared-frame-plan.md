# PFO-4d1p Subsurface RDG Prepared Frame Plan

## 状态

- 日期：2026-08-27
- 当前状态：`source_implemented_static_checks_passed_dynamic_validation_pending`
- 范围：`advanced_lighting/subsurface_pass` 的每帧参数准备、图资源生命周期与 WGPU 上传路径

## 结构重审结论

当前 SSS 热路径在 setup/scatter 两个 executor 中重复解析 profile table、重复计算 camera inverse view-projection，并在每个启用帧创建 3 个 uniform buffer。setup 还绕过帧提交事务直接调用 `queue.write_buffer` 重置 indirect args。局部替换这一次 queue write 不能解决重复 producer、每帧 native resource create 和多 camera 并发覆盖问题。

Unreal `PostProcessSubsurface.cpp`、`SubsurfaceTiles.cpp` 的可迁移边界是：tile/indirect/parameter 资源归 RDG，CPU 只准备一次只读参数产物，GPU ordered clear/build 生成 indirect work，pass 消费图声明的资源。Zircon 已有 transient graph buffer、提交完成后回收、pass-owned `WgpuBufferUploadBatch` 和拓扑有序 command buffer，因此不新增 feature 私有 queue 或进程全局 raw WGPU cache。

## 实施设计

1. 新增 `sss.params` 与 `sss.profiles` built-in graph buffer，容量分别匹配 80-byte params 与 16 x 32-byte profile table，usage 为 `UNIFORM | COPY_DST`。
2. setup pass 是两个参数资源的唯一 producer，scatter pass只读；原 tile/indirect write-read 边保持不变。
3. setup executor 构造唯一 `PreparedSubsurfaceFrame`：一次 resolve profile table、一次计算 inverse view-projection、一次打包共享不可变 payload，并发布两段 `WgpuBufferUpload` 到帧上传批次。
4. indirect args 使用 setup command encoder 上的 `clear_buffer`，首个 setup workgroup 写回固定的 Y/Z dispatch extent，classifier 仅原子累加 X；不再直接写 queue，clear 与 classifier dispatch 保持命令顺序。
5. setup/scatter bind transient graph params/profile buffers；删除每帧 `create_buffer_init`、scatter profile clone 和第二次 resolve/matrix inversion。
6. transient graph pool负责 in-flight 隔离与提交完成回收；不在共享 executor/pipeline cache 中保存跨 camera 可变 prepared 状态。

## 验证与量化门槛

- 静态：SSS production path 中 `queue.write_buffer` 为 0，`create_buffer_init` 为 0，profile resolver 调用为 1，matrix inversion调用为 1。
- 图契约：setup 声明写 params/profile，scatter 声明读 params/profile；两个 buffer 具备精确容量和 `UNIFORM | COPY_DST` usage。
- 稳态目标：资源池热身且提交回收正常时 params/profile native buffer create 为 0；每启用 camera 上传 592 bytes；feature-off prepare/create/upload 为 0。
- 动态 WGPU、RenderDoc、截图、GPU p50/p95/p99 与功耗数据继续服从统一验收阶段，不以静态结果冒充动态证据。

## 当前完成项与静态结果

- 已新增 `PreparedSubsurfaceFrame`，setup 对每个启用 camera 只执行一次 profile table resolve、一次 inverse view-projection 与一次 592-byte immutable payload pack；payload 以两个 source range 进入 pass-local `WgpuBufferUploadBatch`。
- 已新增 `sss.params` / `sss.profiles` 图资源，精确容量为 80 / 512 bytes，native usage 为 `UNIFORM | COPY_DST`；setup write 与 scatter read 形成显式版本依赖，transient graph pool承担 in-flight 隔离和复用。
- 已删除 SSS production path 的 1 次 raw `queue.write_buffer` 和每帧 3 次 `create_buffer_init`；scatter 不再 clone profiles 或重复解析/求逆。
- indirect args 由 encoder ordered clear 清零，首个 setup workgroup 恢复 indirect Y/Z=1，X 继续由 active tile classifier 原子累加，保持原 `[x, 1, 1]` dispatch ABI。
- 静态计数：resolver 1、matrix inverse 1、raw queue write 0、`create_buffer_init` 0、profile clone 0、prepared-frame producer 1、frame-upload append 1。scene-renderer 非测试 raw buffer write 从 7 次 / 7 文件降为 6 次 / 6 文件，剩余均位于 UI 子系统。
- `rustfmt --check`、`git diff --check` 和资源/producer/source 结构审查通过。按当前里程碑执行策略，本切片未运行 Cargo 或动态 WGPU；不得据此宣称稳态 native create=0、性能瓶颈消失或视觉验收通过。
