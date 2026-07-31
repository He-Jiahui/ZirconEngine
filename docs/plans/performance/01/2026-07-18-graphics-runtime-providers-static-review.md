---
related_code:
  - zircon_runtime/src/graphics/runtime_provider
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider
  - zircon_runtime/src/graphics/particle_runtime_provider
  - zircon_runtime/src/graphics/solari_runtime_provider
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider
tests:
  - current graphics runtime-provider slice 32 of 32 Rust files reviewed, 1619 lines
  - all 16 tests read; two capacity source guards and two overflow-behavior regressions added
  - filtered HGI/VG readback projection capacity gates changed from RED to GREEN
  - scoped rustfmt, source contracts and diff check passed
  - current-source Cargo, F2 multi-camera/provider-reload trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics runtime provider静态审查（2026-07-18）

## 当前源覆盖

五个provider目录当前32/32个Rust文件、1,619行已逐文件静态阅读，16条测试已读：`runtime_provider/**` 5/5、`hybrid_gi_runtime_provider/**` 10/10、`particle_runtime_provider/**` 3/3、`solari_runtime_provider/**` 3/3、`virtual_geometry_runtime_provider/**` 11/11。覆盖registration/priority/Arc provider identity、borrowed prepare input、runtime state/update/stats、owned prepare output/feedback、HGI/VG GPU completion投影、VG automatic extract及Solari capability report。

## 直接止损

HGI cache entry与VG assignment/replacement从neutral readback投影到runtime DTO时原用`filter_map().collect()`；由于过滤迭代器size hint下界为0，结果Vec无法利用已知输入记录数，较大readback会反复扩容。本轮先加入容量源码守卫，再把三个投影改为`Vec::with_capacity(input.len())`加单遍push。`u64 -> u32`失败记录仍被跳过；两条混合有效/溢出输入行为测试锁定旧语义。输出顺序、empty-completion判定与payload所有权不变。

## 剩余根因

HGI/VG `prepare_frame`通过viewport record中的动态runtime state执行，当前调用链持有framework state可变借用；任一provider的大mesh/light/page join、分配或插件回调都会延长全局state锁。Solari availability同样在state锁内动态调用。automatic VG已先clone provider Arc并释放锁，但仍按camera把完整mesh slice交给provider，并允许同步`load_model_asset` closure；stable multi-camera可能重复扫描、asset lookup/load与extract build。继续归PERF-MVP-379/414与409：provider catalog按generation冻结，CPU prepare在有界single-flight lane产出immutable artifact，render-owner phase只消费ready handle，模型走resident asset generation，锁不跨provider或I/O。

HGI/VG/particle readback和feedback DTO仍是owned Vec，producer/renderer/sideband/runtime/debug/stats之间缺少唯一sealed generation owner；本轮只清除已知容量增长，未改变公开Clone合同或reload语义。继续归PERF-MVP-415：固定in-flight staging ring、producer边界唯一merge、Arc ticket共享、age/drop/backpressure与旧generation丢弃。

本地Bevy `render_asset.rs`把added/modified资产作为当帧delta提取，prepared资源跨帧驻留，并用`RenderAssetBytesPerFrame`限制上传量。采用“change generation + persistent prepared owner + per-frame budget”的原则，不复制其ECS类型或调度API。

## 验收状态

静态、四条RED→GREEN回归代码、rustfmt、source contract与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，16条测试没有current-source执行结果；RenderDoc CLI不可用且本切片没有capture。providers 0/1/16、cameras 1/8/64、meshes/pages/feedback 0/1k/100k/1M、stable/1% change/reload/failure的callback wall、state-lock hold、asset load、Vec alloc/growth、readback copy/merge、ticket age/drop与RSS未完成，继续留在`pending.md`，不进入`review.md`。
