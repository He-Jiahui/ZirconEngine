# Render Asset Device Generation Recovery Design

## 状态

- 日期：2026-08-27
- 范围：09D semantic render-asset residency 与中立 RHI device generation 的恢复边界
- 当前状态：`residency_slice_implemented_static_checks_passed_dynamic_validation_pending`
- 验收限制：本记录不代表 Cargo、真实 WGPU、device-loss 注入、RenderDoc、性能、功耗或截图通过。

## 重审结论

当前 `RenderAssetResidencyManager` 已用 device/generation 限定 residency ticket、submission ticket、resource handle 与 poll receipt，但 device recreate 只有拒绝旧 receipt 的能力。旧 generation 的 active artifact、pending upload、detached upload、retirement backlog 和 last poll cursor 没有统一失效事务。直接让新 device 继续 poll 会永久保留旧句柄；放宽 receipt 校验则会把旧 submission 状态混入新流，两者都违反 09A 的 generation-local lifetime 约束。

UE 参考边界是 `FRenderResource::ReleaseRHIForAllResources` 与后续 `InitRHI`：全局 owner 先让旧 RHI 资源统一离开可用状态，再由仍存活的 CPU/resource owner 重建，不由 draw/upload 调用点分别容忍失效对象。Zircon 不复制 UE 的全局可变资源表，但保留同一责任顺序：产品 device owner 终止旧代际准入和 submission，residency owner 原子撤销旧 GPU 投影，随后按当前 catalog/readiness 为仍有引用的资源重发新代际请求。

## 事务设计

`recover_device_epoch(failed, replacement, management, readiness, demand_generation)` 必须满足：

1. `failed != replacement`；同一 logical device 的 replacement generation 必须严格递增；manager 中所有 pending/active ticket 以及已绑定 GPU stream 必须属于 `failed`。混合、回退或外来代际 typed 拒绝。
2. 先按稳定 `ResourceId` 顺序收集全部 live entry，解析新代际 ticket seed，并一次性预留 ticket id。任一 catalog/readiness/identity/ticket 预检失败时，不消费 id、不改 entry、不清 GPU 状态。
3. 提交阶段保留 reference count；旧 pending/active 产生原有 typed release，旧 pending upload、active artifact、detached upload、ready retirement 与 submission frontier 统一失效，禁止交给 replacement device destroy。
4. 每个仍有引用的资源发布一个 `QueuedIo` replacement ticket；旧 ticket 不再可推进，active artifact 立即不可见。
5. completion owner 的 bound epoch 改为 replacement，last poll receipt、frontier 与 scratch 内容重置。第一次 replacement receipt 从新流建立严格单调关系；未显式恢复时，新代际 receipt 继续 fail-closed。
6. 旧 native registry 仍由 failed `WgpuRenderDevice` generation owner 统一 drop；residency report 只量化被放弃的句柄引用、allocation bytes、请求和 release，不伪造 native memory 已回收。

## 复杂度与性能约束

- 恢复事务：`O(N log N)` 时间、`O(N)` 临时内存，其中排序用于确定性 ticket/release 顺序；只允许发生在 device recovery 冷路径。
- 稳态维护：仍为有界 `O(K log N)` submission frontier + `O(R)` retirement budget；不增加 entry 全表扫描、每帧 allocation 或第二份 stable-key map。
- GPU 物理 byte 预算继续唯一归 RHI `GpuMemoryBudget`；residency 的 abandoned bytes 只是恢复诊断，不能作为 native release 证明。

## 源码验收入口

- active + uploading entry 在恢复后保留引用计数、发布 replacement ticket，并对旧状态各产生 exactly-one release。
- foreign failed epoch 与 unchanged epoch 在预检阶段原子失败，ticket id 不前进。
- bound completion stream 未恢复时拒绝 replacement submission/receipt；显式 reset 后清 last receipt 并接受 replacement stream。
- 结构扫描锁定恢复逻辑位于 `render_asset_residency/manager/device_recovery.rs`，不把 `manager.rs` 推回 800 行以上。

## 后续动态验收

托管验证阶段补跑 focused Cargo tests，并用 synthetic device-loss fixture 验证 WGPU fault gate terminalize、旧 `WgpuRenderDevice` drop、新 device generation 重传与首帧恢复。真实产品验收必须同时保存 RenderDoc capture、GPU/CPU timing、memory/power 数据和非纯文本 PNG 到 `docs/tests/runtime/render`；在此之前状态保持 source-only。
