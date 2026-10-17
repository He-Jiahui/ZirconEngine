# Compiled AnimationRuntime：相等值和无效写入避免 World mutation

状态：两文件实现与源码清单已冻结；正式scoped静态检查和独立源码复核通过；证据包独立复核通过；同源原生、性能与产品验收仍待实际回执。

修复五个 scalar 与六个 bool 字段在相等判断或转换失败前进入 World::get_mut 的问题。现在先 immutable 确认组件存在，再按原合同转换并用精确相等判断；仅真实变化时取得 mutable 引用并赋值。

字段范围为 AnimationPlayer 的 playback_speed、time_seconds、weight、looping、playing；AnimationSequencePlayer 的 playback_speed、time_seconds、looping、playing；AnimationGraphPlayer 和 AnimationStateMachinePlayer 各自的 playing。局部 FnOnce 读写闭包保留静态分派；没有新增 World API、trigger、parameter 或 owner。

保持外层 stale 优先、缺组件早于转换，operation 仍为 write compiled animation runtime property。Scalar 仅接受 Scalar、Integer as f32、Unsigned as f32 并拒绝非有限值；Bool 仅接受 Bool，Enum 继续拒绝。保留原 ==、正负零相等、微小有限差异和此前 NaN 的有限值修复语义，不增加 clamp 或范围限制。

真实变化保留原有 generation、node dirty 与 inspection invalidation 行为；相等和错误路径不进入 mutable World。新增源码回归要求真实变化恰好 +1 generation；node_records 只检查投影一致性，不能当作 cache rebuild/invalidation counter 证据。

保留前一批 Camera/Mesh 后全部66642B测试前缀，SHA256 `86e6286b6e1d93f127f36d4df953d8832eed955015de27016aacdd15b1bbeb96`；原27 ordinary保留，新增5项，静态源码清单共32 ordinary/0 ignored。公开 sequence 回归经过 compile_animation_sequence、compile_sequence_for_world、apply_compiled_sequence_to_world，要求首次变化+1、再次相同采样不变且 applied_tracks=1。上述是回归要求，尚非原生运行通过。

实际格式证据覆盖 production 整文件与新增 cases suffix；旧 cases 前缀原样保留，不宣称旧整文件格式化通过。核心冻结读回8452f4/0f7567均实际退出0，root原始哈希读回fea5d8退出0。失败与UNKNOWN历史会保留在证据包中，不据此关闭进程或清理状态。

本11字段没有专项原始绝对P95 producer。Runtime170 的101样本/16384keys与 compiled P95*4 < raw P95 仅覆盖 channel sampler，不能证明 World writer P95。Runtime63 G47/G48、08c、170的规模、allocation/work、分阶段耗时与soak门保持OPEN；原所有合批原生、性能、资源门和[冻结Runtime03绝对P95预算](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-absolute-p95-criteria.json)继续AND验收。

新增普通测试源码清单：

- `scene::world::compiled_binding::tests::compiled_animation_runtime_writes_change_and_repeat_all_fields_without_false_generations`
- `scene::world::compiled_binding::tests::compiled_animation_runtime_writes_reject_invalid_values_before_mutation`
- `scene::world::compiled_binding::tests::compiled_animation_runtime_writes_prioritize_missing_components_and_stale_targets`
- `scene::world::compiled_binding::tests::compiled_animation_runtime_writes_preserve_scalar_bool_coercion_and_exact_float_equality`
- `scene::world::compiled_binding::tests::compiled_sequence_repeated_animation_runtime_sample_preserves_world_generation`

源码与主计划：

- [zircon_runtime/src/scene/world/compiled_binding/animation_fields.rs](../../../../zircon_runtime/src/scene/world/compiled_binding/animation_fields.rs)；after 11684B，SHA256 `a281f84cc11583fca1e4ec75f0baf072e0ca63ec5c02427caefe2e643b3268f3`。
- [zircon_runtime/src/scene/world/compiled_binding/tests/cases.rs](../../../../zircon_runtime/src/scene/world/compiled_binding/tests/cases.rs)；after 87470B，SHA256 `b0af7f688310f8beb1d25a62184a54fffd9956b9f115c010a838e8860f27cb78`。
- [docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor.md](63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor.md)；SHA256 `6bf6f9d92d392398df483d7ef68747a32495f705539ed3d7f1740e074a4da1e5`。
- [docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md](08c-animation-runtime-review.md)；SHA256 `74c11de628f370303d3b588335835429e0fb38319ab6173fbe57dcfa26cd1d41`。
- [docs/plans/optimize/zircon_runtime/170-runtime-animation-current-working-tree-source-compiled-pose-skinning-ik-root-motion-event-editor-boundary-review.md](170-runtime-animation-current-working-tree-source-compiled-pose-skinning-ik-root-motion-event-editor-boundary-review.md)；SHA256 `c0943e835b7ab0af1894131e5e09c23e2a2d715cabd56e2a915de4d6101fe99e`。
- [docs/plans/optimize/zircon_runtime/170/2026-09-01-compiled-sequence-channel-sampling-hard-cut.md](170/2026-09-01-compiled-sequence-channel-sampling-hard-cut.md)；SHA256 `1e5e8a853d549e30167b9cfa07a4306cc8280db88b57f21867cbc43a47001e56`。

对应[Astra完成清单](../../astra/features/runtime/2026-10-14-runtime-compiled-animation-noop-generation-completion-list.md)。独立源码复核实际读回均退出0，正式scoped change-guard实际退出0、0errors/0warnings；证据包独立复核亦通过，22个只读witness和4份主计划原始哈希均无差异；两文件增量patch未重放历史修改。源码已应用，不重放旧Light/Camera或累计patch。本轮未创建编译产物、缓存、原生夹具或源码副本，实际释放0B，保留必要归属与失败证据。


最终证据包与静态检查（源已应用，patch仅记录本两文件增量）：

- [runtime-compiled-animation-noop-generation.patch](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-compiled-animation-noop-generation.patch)；32199B，SHA256 `0ef7e3d23d39801dde8aa52659f51610d36a5de68352bc4945d79c4cb795f241`。
- [runtime-compiled-animation-noop-generation-evidence.json](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-compiled-animation-noop-generation-evidence.json)；47336B，SHA256 `e21cbffdb6fbc2a13014837dcf4758fe62b4070750b4fc718c3eebad0fc7cb16`。
- [runtime-compiled-animation-noop-generation-static.json](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-compiled-animation-noop-generation-static.json)；920B，SHA256 `ecd2f9b68404ccfc32b443e00b50663203dcaf82973841d46f9c56d4f390689a`。
