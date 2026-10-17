# Runtime：Compiled Light 相同值写入的 generation 修复

状态：`source_reviewed / native_performance_pending`。核心代码与决策为 gpt-6.1-sol max，独立静态复核为 gpt-6-luna max。

[Runtime63](63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor.md)保留 World generation、反射和 inspection 的唯一 mutation authority；[Runtime71](71-runtime-scene-light-directional-point-spot-rect-photometry-layer-shadow-cookie-ies-extract-authoring-product-integration-review.md)将灯光持久属性交给 Scene owner。本修复针对 CR-R02-runtime_world_property_binding-0002 的真实生产写入：19 个 Light 字段先取得 World.get_mut 再比较，导致相同值返回 false 时 generation 和 mutation effects 已改变。生产 animation/sequence 使用同一 compiled property writer，不能仅修测试或回滚 generation。

[Light writer](../../../../zircon_runtime/src/scene/world/compiled_binding/compiled_scene_light_writer.rs)先执行原有 typed conversion/finite validation，再经不可变 World.get 比较；相同值直接返回 false。仅实际变化才进入原 get_mut、赋值与 node cache dirty 路径。公共 World API、ownership 与 mutation tracking 保持原合同，没有 unsafe、generation 回滚或额外 mutation authority。错误类型/顺序、数值 coercion、NaN/Inf 拒绝、stale writer 拒绝和缺失组件语义均保留。

[真实公开写入回归](../../../../zircon_runtime/src/scene/world/compiled_binding/tests/cases.rs)验证 changed/equal 的返回值和 generation；覆盖19字段，并保留已有 camera/light alias 的重复 intensity=2.5 回归。coerced-equivalent、非有限值、类型错误和 stale writer 继续检查不得产生虚假 mutation。Camera/mesh 的生产修复不在这两个源文件的授权范围。

| 源码 | 字节数 | SHA256 |
| --- | ---: | --- |
| `zircon_runtime/src/scene/world/compiled_binding/compiled_scene_light_writer.rs` | 14755 | `3eddd870204f6767223c0e0f649535a33fef6f695ecc2fc7bd8a6a9650f5d1ad` |
| `zircon_runtime/src/scene/world/compiled_binding/tests/cases.rs` | 40632 | `499b13837b187e81ed2fe4106f7eeffbf7a97dd1d05fc3a87c140117121006b2` |

[补丁](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-compiled-light-noop-generation.patch)、[核心证据](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-compiled-light-noop-generation-evidence.json)和[静态 guard](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-compiled-light-noop-generation-static.json)均已与最终字节校验，核心格式/diff 与 root guard 实际退出0，独立复核无静态阻断。待后续同一批原生验证选择的 ordinary 名称：

- `scene::world::compiled_binding::tests::compiled_descendant_name_index_omits_root_and_preserves_hierarchy_order`
- `scene::world::compiled_binding::tests::compiled_descendant_name_index_stays_current_after_transform_mutation`
- `scene::world::compiled_binding::tests::compiled_descendant_name_index_ignores_unrelated_topology_and_name_changes`
- `scene::world::compiled_binding::tests::compiled_descendant_name_index_stales_for_name_hierarchy_and_topology_changes`
- `scene::world::compiled_binding::tests::compiled_descendant_name_index_stales_when_root_id_is_reused`
- `scene::world::compiled_binding::tests::replacement_world_advances_scene_binding_generation_past_reused_entity_ids`
- `scene::world::compiled_binding::tests::raw_hierarchy_component_mutation_invalidates_cached_roots`
- `scene::world::compiled_binding::tests::compiled_scene_property_target_interns_ids_and_stales_for_its_path_changes`
- `scene::world::compiled_binding::tests::compiled_scene_property_writer_reads_without_path_dispatch_and_requires_rebind_after_name_change`
- `scene::world::compiled_binding::tests::compiled_scene_property_writer_writes_without_string_dispatch`
- `scene::world::compiled_binding::tests::compiled_scene_property_writer_writes_mesh_morph_weights_without_path_dispatch`
- `scene::world::compiled_binding::tests::compiled_scene_property_writer_rejects_morph_weight_index_overflow`
- `scene::world::compiled_binding::tests::compiled_scene_property_writer_writes_camera_and_light_aliases_without_path_dispatch`
- `scene::world::compiled_binding::tests::compiled_scene_property_writer_writes_dynamic_fields_and_ignores_unrelated_schema_changes`
- `scene::world::compiled_binding::tests::compiled_undeclared_dynamic_writer_stales_when_schema_catalog_becomes_restrictive`
- `scene::world::compiled_binding::tests::compiled_scene_property_target_stales_when_root_id_is_reused`
- `scene::world::compiled_binding::tests::compiled_scene_property_writer_stable_access_avoids_text_and_entry_work`
- `scene::world::compiled_binding::tests::compiled_scene_property_writer_scales_stable_access_without_text_work`
- `scene::world::compiled_binding::tests::compiled_scene_property_access_diagnostics_reset_for_clone_and_deserialize`
- `scene::world::compiled_binding::tests::compiled_light_writes_change_and_repeat_all_fields_without_false_generations`
- `scene::world::compiled_binding::tests::compiled_light_writes_reject_invalid_values_before_mutation`
- `scene::world::compiled_binding::tests::compiled_light_writes_preserve_scalar_coercion_and_exact_float_equality`

- [x] 19字段不可变比较先于 mutable World access，相同值不推进 generation。
- [x] 实际改变仍经过原 mutation/cache 路径，保留 finite/type/coercion/stale 合同。
- [x] 最终源码/packet、格式/diff、scoped guard 与独立静态复核完成。
- [ ] 后续合批 exact-source 原生回归、required name/计数与 full-input seal 通过。
- [ ] 全部适用原性能门槛、Runtime03绝对预算与资源上限通过。
- [ ] 根据实际回执关闭历史 BUG 和整体计划验收。

源级 no-op 修复不代表已测得 CPU/P95/RSS 改善。没有本地 native/performance/build/cache/原生夹具或源码副本；实际释放0B，保留必要证据。原始性能标准不放宽，批量交接后继续独立任务，不等待或持续监控编译。
