# Compiled Camera/Mesh 重复写入 generation 修复记录

日期：2026-10-14。状态：代码、静态检查和独立源码复核完成；原生测试、产品与性能验收待 Jenkins 批量结果。

对应计划：[Runtime63](63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor.md), [Camera37](37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md), [Camera99za](99za-runtime-camera-endpoint-director-rig-controller-blend-shake-cut-history-multiview-product-integration-current-source-review.md), [Mesh69](69-runtime-mesh-static-mesh-skeletal-mesh-submesh-lod-instancing-skinning-morph-collision-streaming-product-integration-review.md)。适用合同标识 RSR-G47/G48、CAM-G38、MESH3D-G46/G47/G48。

Camera 三类和 Mesh 六类 compiled property 写入先以不可变组件读取比较新旧值，仅实际变化进入原有可变 World 写入及 cache/inspection dirty 分支。相同值不增加 generation；实际变化保留一次 generation 更新。

错误顺序保持 outer stale 优先、缺失组件先于值转换。保留 Scalar/Integer/Unsigned 原数值转换、有限数检查及精确浮点相等语义，不新增范围限制。+0/-0 相等，微小非零变化仍更新，原 NaN 可由有效值修复。

Morph 已有元素相同值属于 no-op；缺失元素即使写入零也会实际扩容、补零并产生一次变化。checked_add 在可变借用前检查；原地 try_reserve_exact、resize 和赋值行为保留。分配预留失败仍返回原 InvalidPropertyIndex，并保留此前已触发的 generation/component 效果；本次不宣称分配失败完全回滚。

新增五个通过 public compiled write 入口的回归，覆盖所有 Camera/Mesh 字段的 changed/equal、非法值、缺失组件/stale 优先级、数值转换与精确浮点语义、morph 扩容和索引错误。原有 40632 字节 tests/cases.rs 前缀及 22 个普通测试保留，最终命名空间 27 ordinary / 0 ignored。测试代码已编写，尚未取得原生运行通过证据。

- `scene::world::compiled_binding::tests::compiled_camera_mesh_writes_change_and_repeat_all_fields_without_false_generations`
- `scene::world::compiled_binding::tests::compiled_camera_mesh_writes_reject_invalid_values_before_mutation`
- `scene::world::compiled_binding::tests::compiled_camera_mesh_writes_prioritize_missing_components_and_stale_targets`
- `scene::world::compiled_binding::tests::compiled_camera_mesh_writes_preserve_numeric_coercion_and_exact_float_equality`
- `scene::world::compiled_binding::tests::compiled_mesh_morph_writes_preserve_growth_and_index_error_contracts`

测试中的 node_records() 一致性只能证明投影结果；该接口每次重新投影，因此不把输出相等作为 cache hit 或 cache invalidation 计数证明。实际变化继续使用原可变访问及脏标记路径。

源码归属与证据：

- [zircon_runtime/src/scene/world/compiled_binding/property_path/write.rs](../../../../zircon_runtime/src/scene/world/compiled_binding/property_path/write.rs)；before 10139 B / `1e4f0b6ebed5327f2ee680fdee447c37d89ff16cf28f4b3a6ec9d5b90de7dc0d`；after 11289 B / `97c26daef47155f950f43c26a0f4ccb30bfa09e6b0cb0f68ffbc2b501a7e6d50`。
- [zircon_runtime/src/scene/world/compiled_binding/tests/cases.rs](../../../../zircon_runtime/src/scene/world/compiled_binding/tests/cases.rs)；before 40632 B / `499b13837b187e81ed2fe4106f7eeffbf7a97dd1d05fc3a87c140117121006b2`；after 66642 B / `86e6286b6e1d93f127f36d4df953d8832eed955015de27016aacdd15b1bbeb96`。
- [runtime-compiled-camera-mesh-noop-generation.patch](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-compiled-camera-mesh-noop-generation.patch)；34551 B / `45df0a36151d24d067c62fb5d5d3385ab2b996f73d90badb94b55d1b0db234ef`。
- [runtime-compiled-camera-mesh-noop-generation-evidence.json](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-compiled-camera-mesh-noop-generation-evidence.json)；28165 B / `72d01bb11a975f36969c725a2c0138ea365d90e2c60c83cec541372b76ccec88`。
- [runtime-compiled-camera-mesh-noop-generation-static.json](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-compiled-camera-mesh-noop-generation-static.json)：exact2 scoped change guard，subprocess/outer terminal 0，errors 0。

五个新增 FQN 在 B15 冻结的 22 项清单中不存在，虽然旧 broad selector 可词法匹配。后续批次普通 Runtime floor 应按实际增量 116→121，required named 59→64；不把旧筛选能力当作测试已存在。历史 Light packet 保持不可变，其 cases/writer witness 由本层 before/after 衔接，不宣称历史哈希仍代表当前全部源码。

这九类属性写入没有原计划定义的独立绝对 P95 或专用 producer。本记录不移用 Camera target/key/sequence/topology 等其他 producer 的百分比门槛，也不宣称已测得 CPU/P95/RSS 收益。原 16 Cargo + 2 Python arms、16 producer 与 legacy 11、原样本矩阵和计数、Runtime44 54 行 RSS 双 128MiB 上限、冻结 Runtime03 绝对 P95 与原 helper 门槛均继续 AND 验收。

- [x] 已实现 no-op generation 修复并保留真实 morph 增长语义。
- [x] 已追加五个有意义的 public-write 回归并保留旧 22 项。
- [x] scoped fmt/diff、exact2 guard 和独立源码复核完成。
- [ ] exact-source 原生 27 ordinary / 0 failed / 0 ignored 及合批全部原测试通过。
- [ ] 原性能矩阵、RSS 与 Runtime03 绝对 P95 均通过。
- [ ] 产品验收与 BUG 闭环完成。

产物仅保留 patch、源码静态证据、guard、计划记录和后续验证交接。根未创建 build/cache/native fixture/源码副本；实际释放空间 0 B。Jenkins 负责其已登记原生产物和进程的结束与清理。

