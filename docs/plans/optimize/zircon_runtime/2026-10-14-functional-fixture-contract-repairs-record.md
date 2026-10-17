# Runtime：TextAtlas 与材质测试夹具合同修复

状态：`source_reviewed / native_performance_pending`。核心决策与修复：gpt-6.1-sol max；独立 delta 静态复核：gpt-6-luna max。

[TextAtlas 测试](../../../../zircon_runtime/src/text/atlas/render_submission/tests/retry_frame/state.rs)的两个队列边界用例原声明 6×6 AlphaMask，却提供 64/48B。AlphaMask 为每像素 1B，源校验先返回 DataLengthMismatch，无法进入本来需要测试的容量阻塞路径。CR-TEXT-RASTER-0001 的两个内容尺寸改为 8×8，保留 64B；CR-TEXT-RASTER-0002 改为 8×6，保留 48B。source helper 同时从该尺寸生成 screen rectangle，因此完整 source/glyph identity 断言仍成立。

所有原预算和数量断言保留：max_pages_per_format=0；第一用例 glyph cap=1、byte cap=128，第二个合法源因 glyph 数溢出；第二用例 glyph cap=2、byte cap=64，48+48=96>64，第二个合法源因字节预算溢出。padding=2 后 12×12/12×10 均小于 32×32 page，避免触发无关尺寸拒绝。

[材质打包测试](../../../../zircon_runtime/src/graphics/scene/resources/tests/resource.rs)的 CR-R02-runtime_wave12_graphics_resource_residency-0001 交换了非零 0.4 弧度旋转的期望。CPU helper 返回 [cos,sin]，values[51]/[62] 与 WGSL data12.w/data15.z 对应字节偏移 204/248；期望分别修正为 cos/sin。256B ABI、角度、偏移、容差、全部其他输入与断言保留，生产 packer/helper/WGSL 没有修改。

| 源码 | 最终字节数 | SHA256 |
| --- | ---: | --- |
| `state.rs` | 13225 | `5234bd18959230e27658cf9e6fe3f4b9fbd7429ae71dd2cdabf548f6bd2afb88` |
| `resource.rs` | 10163 | `4950529191198b62f1d5957094dc94dbf7f69691df227540c93c109744e027bd` |

[补丁](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-editor-fixture-contract-repairs.patch)、[核心证据](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-editor-fixture-contract-repairs-evidence.json)、[静态 guard](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-editor-fixture-contract-repairs-static.json)已与最终字节核对。核心 rustfmt/diff 静态检查及 root guard 实际退出 0，独立复核无静态阻断。TextAtlas/材质文件仍为原 9/10 个 ordinary、0 ignored，仅选择真正修复的三个用例，不把未执行 sibling 计为通过：

- `text::atlas::render_submission::tests::retry_frame::state::render_text_atlas_bitmap_retry_frame_driver_bounds_the_blocked_queue`
- `text::atlas::render_submission::tests::retry_frame::state::render_text_atlas_bitmap_retry_frame_driver_bounds_blocked_queue_source_bytes`
- `graphics::scene::resources::gpu_material_uniform::gpu_material_uniform_resource::tests::standard_material_uniform_packs_clearcoat_normal_without_growing_the_256_byte_abi`

这三例与 B13 适用 arm 的有效 ordinary 选择重叠为 0。第14批在已有 Runtime arm 增加三例，再加 Runtime206 两例，下限 89→94；原 16 Cargo/2 Python、既有 required names、原始 producer 和性能门槛保持。专项优化计划编号尚未验证，不据 BUG ID 猜编号，也不据夹具修复关闭其他计划项。

- [x] 构造有效 AlphaMask 源，保留原队列/字节压力及 identity/overflow 断言。
- [x] 非零旋转 cos/sin 期望与 CPU/WGSL/256B ABI 合同一致。
- [x] 最终源码/packet 校验、静态检查、独立 delta 审查完成。
- [ ] 第14批 exact-source 原生用例、匹配下限及前后 full-input seal 通过。
- [ ] 所有适用原始性能门槛、Runtime03 绝对预算和资源上限通过。
- [ ] 据实际回执关闭历史 BUG 与整体验收。

未运行本地 native/product/performance，不创建编译/缓存/原生夹具、临时脚本或源码副本；保留必要证据，实际释放空间 0B。异步合批交接后继续独立功能修复，不等待或持续监控编译。
