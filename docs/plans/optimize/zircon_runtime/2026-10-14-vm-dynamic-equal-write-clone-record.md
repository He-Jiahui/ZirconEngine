# VM 动态组件同值写入先比较再克隆

状态：源码修复、静态检查和独立源码审查已完成；原生测试、性能与产品验收保持开放。

同值写入此前会先深克隆整个 VM JSON 对象，再返回未变化。本轮在共享的实体、类型、字段权限、组件存在性和对象形状检查之后，先比较目标字段的 JSON 值；相等时直接返回 `Ok(false)`。变化写入仍构建完整候选、校验全部字段，再提交一次，并执行原有 dirty、generation 和 cache 更新。兼容动态组件分支沿用原代码。

公开入口保持原有错误顺序：compiled writer 先检查 stale；有效 writer 随后转换输入，再检查实体、VM 类型、声明及可编辑性、组件实例和对象形状。只读字段即使同值仍拒绝，缺失实例、未声明字段、类型错误和非有限输入的顺序保持。

修改范围与原始字节：

| 文件 | 本轮前态 | 冻结后态 |
| --- | --- | --- |
| `zircon_runtime/src/scene/world/dynamic_components.rs` | 40144 B / `3a53ee3a34810ea667293676d04a169482cc23ca701213324dff9e62f52816b3` | 40194 B / `afebea277c88105f10a73e24cbd320fb3fdb4f8023b1fcaef936ad6eab05dc73` |
| `zircon_runtime/src/scene/world/compiled_binding/tests/cases.rs` | 87470 B / `b0af7f688310f8beb1d25a62184a54fffd9956b9f115c010a838e8860f27cb78` | 104740 B / `9f9b1ad1e67b6e70b077ea48ee1944f5c20e6f1d0479e7e8d463739a3f3e554b` |

补丁只包含本轮前态到后态的增量。前态是已有工作的实际 checkout；历史 Light、Camera、Animation 修改均保留。测试文件原 87470 B 前缀完整不变，原 32 项 ordinary 全保留，新增 4 项，共 36 ordinary、0 ignored。生产文件原 140 条 `//` 注释逐行保留；将定位行 135 误记为数量的历史失败读取已保留。

新增公开 API 回归均由 `compiled_binding/mod.rs` 的 `#[cfg(test)] #[path = "tests/cases.rs"] mod tests` 编入，命名空间为 `scene::world::compiled_binding::tests`：

- `compiled_vm_dynamic_repeated_field_write_preserves_large_component`：含 256 KiB 无关只读字段的组件连续同值写入，完整 JSON、generation、writer current 和节点投影保持。
- `compiled_vm_dynamic_changed_field_write_remains_transactional`：变化写入提交一次，类型与完整候选错误不提交，之后同值写入保持。
- `compiled_vm_dynamic_writer_preserves_error_priority_and_stale_admission`：可编辑性、声明、缺失实例、类型、非有限输入及 stale 优先级保持。
- `compiled_sequence_repeated_vm_dynamic_sample_preserves_world_generation`：真实 compiled sequence 重复同值采样仍计 `applied_tracks = 1`，world generation 稳定；改名后按 stale 计 missing。

源码 writer 实际收敛为 core `7/9a0d99` exit 0、session 30331 已关闭；其 PID 因截断仍为 UNKNOWN。两包 writer 为 core `22/755b64` exit 0、session 87597 已关闭，反向补丁检查子进程 40868 joined/0；root 包读取为 `112/b4ecfc` exit 0。root scoped guard 为 `108/79b665` exit 0，errors 0、warnings 0；独立 Luna 语义审查结论为 `SOURCE_STATIC_PEER_PASS`，其 reverse-check reader `17/805806` exit 0，Git 子进程 51488 joined/0。这些记录证明源码及静态检查，不构成原生或性能验收。

第 17 批现有词法 selector 会匹配新增 4 项，但其 required 集和下限仍是 104 / 161。下一批至少增至 108 / 165；与 Runtime03 失败接管合批时，再按 source inventory 增加该组新旧必需用例，不新增逐用例编译任务。旧 argv、required names、含 null 的 prefix floors、16 Cargo、2 Python、16 producers、独立 legacy #11 和全部原性能/资源门槛继续保留。

本轮只支持“同值写入跳过大组件深克隆”。Scalar 到 JSON 的转换仍可能分配；节点记录投影不是缓存计数器。尚无本轮 clone counter、完整 writer P95 或原生结果；Runtime170 的 channel sampling producer 不能替代 World writer 性能证据。Runtime63 的 RSR-G47/G48、08c allocation/work/sample/scale/soak 和 Runtime170 原门槛继续开放，完整编译前后输入封印及实际测试收据仍必需。

原计划与证据：[63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor.md](63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor.md)、[08c-animation-runtime-review.md](08c-animation-runtime-review.md)、[170-runtime-animation-current-working-tree-source-compiled-pose-skinning-ik-root-motion-event-editor-boundary-review.md](170-runtime-animation-current-working-tree-source-compiled-pose-skinning-ik-root-motion-event-editor-boundary-review.md)、[2026-09-01-compiled-sequence-channel-sampling-hard-cut.md](170/2026-09-01-compiled-sequence-channel-sampling-hard-cut.md)；[增量补丁](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-vm-dynamic-equal-write-clone.patch)、[源码证据](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-vm-dynamic-equal-write-clone-evidence.json)、[静态报告](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-vm-dynamic-equal-write-clone-static.json)；[Astra 完成列表](../../astra/features/runtime/2026-10-14-runtime-vm-dynamic-equal-write-clone-completion-list.md)。

生命周期：本轮仅保留源码、补丁、证据、静态报告及两份记录。root 未创建编译产物、缓存、夹具、临时 helper 或源码副本；实际释放 0 B。历史 UNKNOWN 进程及其他会话产物保持，不声明全部清理完成。
