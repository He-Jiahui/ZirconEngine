# SVG 与 GPU 图像驻留审查及优化计划

日期：2026-08-25
审查基线：`1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8` 加当前共享工作树
状态：设计已收敛；当前相关生产路径存在外部未提交重构，本报告不抢占其所有权；产品性能验收尚未执行

## 1. 结论

用户观察到的“SVG 反复加载”不能只用一个 cache 解释。当前路径至少有六个独立阶段：

1. 逻辑 brush/icon 名称解析为候选资源路径；
2. 文件读取与 SVG parse，得到矢量树；
3. 按物理尺寸、DPI、tint 栅格化为 RGBA 产品；
4. 小图标装入 CPU atlas 页；
5. 命令流按 `(resource_key, generation)` 携带资源或仅携带句柄；
6. RHI 将未驻留产品上传为 WGPU texture，并在后续帧复用。

当前工作树已经有 CPU 像素 cache、SVG tree cache、CPU icon atlas、命令资源表、WGPU image cache 和跨窗口 shared image registry。源码方向已经从“每次 paint 读取 SVG”推进到资源驻留模型，但尚不能宣布问题解决：相关文件均有外部未提交改动，且没有 current-source 产品采样证明稳定交互帧的 SVG miss 和 GPU upload 都归零。

真正的目标不是“加一个 HashMap”，而是建立一个跨层资源身份合同：内容未变时，逻辑资源、栅格产品、atlas 页和 GPU texture 的身份都必须稳定；只有内容、栅格规格或设备代际变化时才允许生成新产品。

## 2. 当前源码事实

### 2.1 CPU 资源与 SVG 层

`zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/cache.rs` 当前包含：

- 4,096 项、64 MiB 的成功/失败像素 cache；
- LRU 顺序；
- base key 到具体 raster key 的反向索引；
- source path alias 与内容指纹；
- 指定路径失效、全量资源刷新对账和异步 epoch 防陈旧提交；
- `Arc<[u8]>` 像素共享，cache hit 不再深拷贝整块 RGBA。

`visual_assets/svg/cache.rs` 当前缓存解析后的 SVG tree，并公开 memory hit/miss 计数。`loading/async_loader.rs` 提供后台加载、in-flight 去重和完成唤醒。这里已经覆盖“同一个 SVG 每次 paint 都重新读盘和 parse”的直接修复方向。

但仍需产品证明：

- candidate key 是否在相同图标、相同 DPI 和相同 tint 下稳定；
- miss 是否只发生在第一次使用或显式失效后；
- resize 期间目标尺寸取整是否抖动，造成连续 raster key；
- 文件监听的重复路径事件是否只做指纹核对而不会清空无关产品；
- 全局 `Mutex` 是否在高频并行窗口 paint 中形成串行等待。

### 2.2 CPU atlas 与命令流层

`zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/icon_atlas.rs` 当前包含：

- 小于等于 64 px 的 editor icon 入 atlas；
- 64 页、64 MiB 上限；
- resource key + source generation + raster size 的 slot 身份；
- 页级 resource key 和 generation；
- 页 LRU 替换；
- atlas UV 投影。

命令流随后把重复 image payload 压缩到 image resource table。GPU presenter 在构建命令流时先调用 `is_image_resource_resident(resource_key, generation)`，已驻留资源不应再次携带 RGBA 上传体。

命令流内部的资源组合并此前通过`into_entries()`把`HashMap<resource_key, BTreeMap<generation, resource>>`展平，再为每个generation克隆一次完整`resource_key`。这不会触发SVG parse或GPU upload，但会让包含G个generation的合并批次产生G次字符串堆分配。当前独立候选直接消费outer key并`BTreeMap::append`整个generation map：时间仍为`O(G log G)`的map合并语义，key分配由G次降为0；相同generation仍由新批次覆盖。源码性能合同先RED后1/1 GREEN，Rust行为回归源码和scoped rustfmt/diff通过，managed Rust与allocator profile尚未执行。

当前 atlas 页在写入后封页，使稳定页 generation 不因后续 icon 加入而反复变化。这个策略牺牲部分装箱密度，但能保持 GPU 资源身份稳定，符合交互优先原则。

剩余风险：

- `pack_editor_icons_into_atlas` 仍遍历当帧 image commands；需要证明 damage stream 将访问量限制在受损区域，或在完整帧中该线性访问仍低于预算；
- atlas 和上游像素 cache 同时保留同一 icon RGBA；
- atlas 页 LRU 替换后必须精确使对应 GPU generation 失效，不能清空全部页；
- 尺寸接近 64 px 或 DPI 临界点时不能在 atlased/non-atlased 两条路径间来回抖动。

### 2.3 WGPU 驻留层

`zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs` 当前包含：

- `HashMap<resource_key, BTreeMap<generation, resource>>` 驻留索引；
- 256 项、64 MiB 的 GPU image cache；
- generation 命中时跳过 `queue.write_texture`；
- LRU admission/eviction；
- invalid payload、上传次数、上传字节、cache hit、prune visit 和 resident bytes 统计；
- draw-list generation 的 prepare 快路径；
- external image 和 shared image registry 解析。

`shared_image_registry.rs` 另有 64 MiB 共享纹理预算，用于相同设备上的多 surface/window 复用。`presenter/gpu/present.rs` 在生成 draw list 前查询驻留性，因此稳定资源理论上只上传一次。

这说明“完全没有 GPU 图像缓存”已不符合当前工作树事实；更准确的问题是：GPU cache 的正确性和命中率尚未由 current-source 产品数据证明，且上游 key/generation 抖动会让一个存在的 GPU cache 看起来等于没有。

## 3. Unreal 对照

主参考为本地 Unreal Slate：

- `dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIResourceManager.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIResourceManager.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHITextureAtlas.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHITextureAtlas.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/SlateResourceHandle.h`

Slate 的关键做法不是在 `OnPaint` 中反复解码图片，而是：

- brush 先解析为稳定的 `FSlateResourceHandle` / shader resource；
- dynamic texture、UTexture、material、atlased texture 分别进入长期 resource map；
- atlas 由 renderer resource manager 拥有并在明确的 atlas update 阶段更新；
- draw element 消费 resource handle/proxy，而不是携带一份新的源 SVG 或 RGBA；
- 资源创建、atlas 更新、绘制提交属于不同生命周期。

Zircon 应保持当前 `(resource_key, generation)` 句柄化方向，但需要继续收束所有权：上游负责形成不可变 raster product，atlas/resource manager 负责驻留，paint command 只引用产品，RHI 负责 device-local texture 生命周期。

## 4. 结构性问题

### 4.1 多层各自 64 MiB，不等于一个 64 MiB 总预算

当前至少存在以下独立上限：

| 层 | 当前上限 | 可能持有内容 |
| --- | ---: | --- |
| visual asset pixel cache | 64 MiB | 原始 bitmap 或 SVG raster RGBA |
| editor icon atlas | 64 MiB | 再复制一次小图标 RGBA |
| WGPU per-surface image cache | 64 MiB | GPU texture，且 resource 当前可保留 CPU RGBA mirror |
| shared image registry | 64 MiB | 跨窗口共享 GPU texture |

因此单层有界并不能证明进程 RSS 合理。相同图标可能同时存在于 raster cache、atlas page、per-surface CPU mirror、per-surface texture 和 shared texture。后续必须以产品身份做跨层内存账，并决定唯一 CPU owner；不能继续仅提高各层预算。

### 4.2 `resource_key` 与 `generation` 是性能合同

稳定帧必须满足：

- `resource_key` 表示内容与 raster 规格，不包含 node id、paint order、frame sequence 或临时路径拼接结果；
- SVG raster key 必须包含规范化 source identity、内容 revision、物理宽高、DPI bucket、tint/variant；
- 相同输入必须产生相同 key；
- `generation` 只在产品 bytes、atlas 页内容或 device generation 改变时增加；
- hover 若只改变颜色，可选择有限 tint variant；不得每帧创建新色值或新 generation；
- resize 的物理尺寸必须量化到稳定整数/bucket，避免亚像素尺寸产生无限产品。

### 4.3 失效必须局部传播

单个 SVG 文件变化时，允许发生：

1. 该 source 对应 SVG tree 失效；
2. 该 source 的 raster variants 失效；
3. 包含它的 atlas 页获得新 generation，或以稳定 slot 做局部页更新；
4. 使用该资源的 commands/regions 标记 damage；
5. 相关 GPU page/texture 上传一次。

不允许发生：清空全部 visual cache、全部 icon atlas、所有窗口资源或全 UI presentation。

## 5. 复杂度目标

设可见 image command 数为 `I`，当帧首次出现的 raster 产品数为 `M`，GPU 未驻留产品数为 `U`：

- 稳定 cache lookup：均摊 `O(1)`，不得读盘、parse、raster 或 upload；
- 稳定命令资源压缩：`O(I)`，payload bytes 为 `O(U)` 而不是 `O(I)`；
- 命令流资源组合并：移动U个资源组及其generation map，resource-key heap allocation为0；
- atlas 新增：只处理 `M`，不重排既有页；
- GPU prepare：扫描可见 image source，上传工作为 `O(U + uploaded_bytes)`；
- 指定 source 失效：`O(aliases + variants + dependent_pages)`，不得 `O(all_assets)`；
- idle frame：SVG miss、visual miss、GPU upload write 和 cache key allocation 全部为 0。

当前 `BTreeMap` 使部分 CPU cache lookup 为 `O(log N)`，在 4,096 项上不是首要瓶颈。只有产品 profile 证明锁竞争或 map lookup 占比显著后才替换容器，避免再次做细节优化掩盖身份抖动。

## 6. 验证矩阵

所有产物写到 `E:\zircon-profiles\...` 或批准的 D/E/F Cargo target；不写 C 盘。Cargo 仅使用仓库官方 managed validation。

### 6.1 必须采集的现有计数器

- `visual_asset_cache_hit_count`
- `visual_asset_cache_miss_count`
- `visual_asset_cache_candidate_build_count`
- `visual_asset_async_enqueued_count`
- `visual_asset_async_deduplicated_count`
- `svg_tree_cache_memory_hit_count`
- `svg_tree_cache_miss_count`
- `gpu_upload_bytes`
- `gpu_image_upload_writes`
- `gpu_image_shared_resolves`
- `gpu_image_shared_upload_writes`
- `gpu_image_cache_key_allocations`
- `gpu_image_cache_prune_visits`
- `gpu_image_cache_admission_rejects`
- `gpu_image_invalid_payloads`
- `gpu_image_cache_resident_bytes`
- `gpu_image_shared_resident_bytes`
- `gpu_image_prepare_command_visits`
- `gpu_image_prepare_cache_hits`

需补充而当前不足的计数：SVG file read bytes/count、SVG parse time、raster time、raster product bytes、atlas page update bytes、per-surface CPU mirror bytes、visual cache lock wait time、按 `(resource_key, generation)` 的 churn top-N。

### 6.2 压力场景

1. 冷启动并打开默认编辑器工作台，记录首次资源形成成本。
2. 预热后 idle 10,000 presents，资源内容不变。
3. 在含相同 SVG 的按钮间 hover/press 往返 10,000 次。
4. 连续 resize 1,200 帧，覆盖 800x600 到 1,920x1,080，再返回原尺寸。
5. 在 100%、125%、150%、200% DPI 间切换，并在每档返回既有尺寸。
6. 打开两个共享同一 device 的 editor window，验证第二窗口 shared resolve 而不是第二次 upload。
7. 修改一个 SVG，再发送重复 watcher 事件；只允许一个内容 revision 生效。
8. 构造超过 atlas/cache 容量的图标集合，验证有界 LRU、无 thrash、无全局清空。

### 6.3 验收阈值

预热完成后的稳定阶段：

- SVG file read = 0；
- SVG tree miss = 0；
- visual asset miss = 0；
- GPU image upload writes = 0；
- GPU upload bytes = 0；
- GPU image cache key allocations = 0；
- invalid payload 和 admission reject = 0；
- 同一逻辑图标每个 DPI/tint/size bucket 只有一个稳定 raster key；
- 返回已使用 DPI/size bucket 时不重新 parse、不重新 raster、不重新 upload；
- CPU/RSS 与 GPU resident bytes 在压力后回落到预算范围，不随 presents 数线性增长；
- pointer/hover p95 不因资源处理超过 8 ms，整帧 p95 保持在 16.7 ms 预算内；
- resize 期间每帧不得出现 SVG file IO，GPU upload 只允许新尺寸 bucket 的首次产品。

上述阈值目前均为待验证，不得写成已通过。

## 7. 实施顺序

### M0：先绑定 current-source 数据

- 等相关外部 SVG/atlas/RHI 改动进入稳定 HEAD；
- 通过 managed validation 构建 current-source editor；
- 采集冷启动、预热 idle、hover、resize、DPI、双窗口和 hot-reload 数据；
- 输出 resource churn top-N，并把 CPU miss 与 GPU upload 分开归因。

### M1：修身份抖动，而不是扩大 cache

若稳定阶段仍有 miss/upload：

- 从首个变化的 `(resource_key, generation)` 反向追到 candidate/raster/atlas；
- 统一路径 canonicalization 和 content revision；
- 对物理 raster size 做明确 bucket；
- 限定 tint variant；
- atlas 页只在 bytes 变化时增加 generation；
- 添加 lower-layer regression，证明同输入跨两帧身份相同。

### M2：收束跨层内存所有权

- 建立 visual/raster/atlas/GPU/shared 的统一 residency telemetry；
- 明确 CPU RGBA 的唯一长期 owner；
- GPU upload 完成且可由上游按 key 重取时，评估释放 per-surface CPU mirror；
- atlas 产品尽量共享上游 `Arc` 或使用页级 staging 生命周期，避免长期双份像素；
- image resource table跨stream合并必须整体移动resource group，不得按generation重新分配key；
- 给整个 UI image subsystem 设置总预算，而不是四个彼此独立的 64 MiB 预算。

### M3：局部失效与设备生命周期

- 单 source revision 只重建其 variants/dependent pages；
- GPU eviction 或 device recreation 后，驻留查询必须返回 false，让上游精确重送对应资源；
- 多窗口销毁不应清空共享 registry 中仍被其它窗口使用的纹理；
- hot reload regression 覆盖重复 watcher event、内容未变事件和真实内容变化。

### M4：产品验收

- 重跑 6.2 的全部压力场景；
- 同时记录 CPU、RSS、allocator、GPU time、upload、draw call、painted pixels 和交互 p50/p95/p99；
- 与本报告阈值及 Unreal 的资源句柄/atlas 生命周期原则对照；
- 只有 current-source managed build、产品交互和内存稳定性都通过后，才关闭“SVG 反复加载/GPU cache”问题。

## 8. 当前所有权边界

本次审查未修改以下生产路径，因为它们已经存在大量外部未提交变更：

- `zircon_editor/.../paint_template_nodes/visual_assets/**`
- `zircon_editor/.../chrome_command_stream/icon_atlas.rs`
- `zircon_editor/.../presenter/gpu/**`
- `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs`
- `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs`

后续集成必须按 hunk/path 归属，不得把这些外部变更吸收到本优化候选。当前可接受产物仅是本报告和此前独立、静态验证通过的优化候选；Cargo/产品验证仍等待受控 lane 与 copy-stable HEAD。

## 9. 2026-08-28 current-source 静态收口

本轮沿完整资源代际链重新核对了 current source：

- SVG raster 的底层身份由宽高和 RGBA 内容指纹组成，图标包装仍保留该 content key：`icon-raster:<content-addressed-key>`；
- vector target 在 pixel-cache lookup 之前进入有界自适应 bucket，稳定 resize bucket 不产生新 raster identity；
- pixel cache hit 在候选路径构造之前返回，SVG tree 的 query-path memory hit 在 metadata/stamp 和 parse 之前返回；
- 指定 source 变化通过内容指纹只移除对应 logical asset variants；异步结果还必须通过 cache epoch，不能把旧产品写回；
- icon atlas 以 source key + generation + size 索引 slot，写入后的 page 立即封存并取得独立 generation，后续新增或内容变化不会改写已经发布的页；
- command stream 对同一 `(resource_key, generation)` 只查询一次 residency，已驻留时移除 RGBA；WGPU 只有 generation 未上传时才进入 `queue.write_texture`，cache 受 256 项/64 MiB 约束。

因此 current-source 架构上已经具备“稳定产品不重复读盘、parse、raster 或上传”的闭环。若真实 Editor 仍观察到重复加载，首要嫌疑不再是缺少 cache，而是 source/target/tint key 抖动、容量驱逐、device recreation 或 watcher 失效风暴；必须从现有 miss/upload 计数和待补的 churn top-N 反向定位，不能再并列增加一套 cache。

新增 `tools/editor_svg_gpu_residency_pressure.py` 将上述合同建模为确定性复杂度证据。默认场景为 10,000 次稳定 present、每次 2,048 个 image command、256 个 SVG source、每个 4 个 raster variant、16 个 256x256 atlas page，并包含一次单 SVG 内容变化：

- 无驻留的逐 command 重建基线为 20,480,000 次 file read/parse/raster/upload；
- retained 路径的冷启动加一次热更新为 257 次 file read/parse、1,028 次 raster 和 17 次 atlas page upload；
- 稳定阶段 file read、parse、raster、GPU upload write/bytes 均为 0；
- 模型中的 file read/parse 减少约 79,688.72 倍，raster 减少约 19,922.18 倍，upload write 减少约 1,204,705.88 倍；这些是工作量比值，不是产品加速比；
- 四个独立 64 MiB 层级预算的配置上界合计为 256 MiB，且仍未计 allocator/container 开销，验证了 M2 统一 residency 预算仍不可省略。

证据产物：`E:\zircon-profiles\editor-svg-gpu-residency-20260828.json`，SHA-256 `75229B1A12EAB306527937F78468A84D02763E6008EAB96A41C1927EE6862F3A`。

静态验证：SVG/GPU 端到端设计合同、压力模型、vector bucket、异步 materialization、atlas borrowed lookup 和 Runtime GPU image cache 共 47/47 通过；Python compile 与 scoped `git diff --check` 通过。相关生产路径已有大量外部未提交改动，本轮没有修改或吸收这些文件，也没有运行 raw Cargo。

本节仍不构成产品性能通过。M0/M4 的 current-source managed Editor 构建、冷/热 SVG counters、GPU upload counters、CPU/RSS、资源 churn top-N 和 hover/resize p50/p95/p99 尚未执行，不能据此宣称用户观察到的卡顿已经消失。
