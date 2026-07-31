---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
tests:
  - current viewport_record Rust source census 16 of 16 files reviewed, 1040 lines
  - seven camera-keyed state maps and their submit cleanup call graph accounted for
  - camera key clone shares both layer stores regression test added
  - scoped rustfmt and diff check passed
  - current-source Cargo, long camera churn, F2 multi-viewport and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics viewport_record静态审查（2026-07-18）

## 当前源覆盖

`zircon_runtime/src/graphics/runtime/render_framework/viewport_record/**`当前16/16个Rust文件、1,040行已逐文件静态阅读，覆盖camera history key、capture/generation/pipeline/quality、history/motion/particle、provider runtimes、product reports、surface lease及record owner。当前13个单元测试均已阅读；本轮新增layer-storage sharing合同，但未执行Cargo。

## 直接止损

`ViewportCameraHistoryKey`的culling/volume layer identity原为两份owned `Vec<RenderLayer>`。同一frame key随后被history、runtime、report、motion与particle更新反复clone，layer越宽复制越多。本轮先增加key clone pointer-sharing RED测试，再把两份storage改为`Arc<[RenderLayer]>`；key构造仍保持完整layer语义（包括超过scene-schema-v1的layer），但跨状态表clone只增加Arc引用，不再深复制layer Vec。

## 状态表根因

`ViewportRecord`当前有7张`HashMap<ViewportCameraHistoryKey, ...>`：camera history、HybridGI runtime、Virtual Geometry runtime、light-grid report、VG debug snapshot、motion-vector camera和particle previous sprites。产品submit调用图在多处构建/clone相同key并分别probe/insert。runtime provider变化会整表clear，viewport destroy只显式forget history；常规camera移除、viewport rect/layer/type变化没有统一active-key reconcile或prune。

由于viewport、culling/volume layer等动态属性属于key identity，连续编辑这些设置会创建新entry；旧history可能持GPU history handle，provider state和debug/particle payload也可能很大，并一直保留到viewport销毁。新增`PERF-MVP-410`要求Render09稳定camera slot与动态validation generation、Runtime07单一camera state table及每帧mark/reconcile/prune；移除slot须调用renderer history cleanup，短暂消失只能使用显式有界TTL。

`SlotLease`通过Drop恢复surface，在state锁外提交后仍能安全归还，当前无额外分配；generation和temporal index为O(1)。capture/compiled pipeline保存的owned对象继续受`PERF-MVP-023/365`约束，不在本切片重复编号。

## 验收状态

局部文件已通过rustfmt、Arc storage源码合同和`git diff --check`。Cargo协调器仍在启动前JSON解析失败；camera 0/1/8/1k、layer/rect churn 1/100/10k、300/10k frame内存/state-entry曲线、F2多视口/history continuity及RenderDoc资源释放均无current-source动态证据。目录继续留在`pending.md`，不进入`review.md`。
