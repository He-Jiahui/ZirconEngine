# Runtime VM 动态组件同值写入完成列表

对应 [优化记录](../../../optimize/zircon_runtime/2026-10-14-vm-dynamic-equal-write-clone-record.md) 与 [源码证据](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-vm-dynamic-equal-write-clone-evidence.json)。本列表区分源码交付与实际验收。

- [x] 同值比较位于完整对象克隆之前；共享 admission、错误优先级、变化候选的完整校验及单次提交保持。
- [x] 原 32 项 ordinary 及 87470 B 测试前缀保留；新增 4 项公开 API / 真实 compiled sequence 回归，合计 36 ordinary、0 ignored。
- [x] 原生产 140 条行注释保留；source/patch/witness 原始哈希与只读 reverse-check 已核对。
- [x] root scoped guard passed，errors 0 / warnings 0；独立 Luna `SOURCE_STATIC_PEER_PASS`。
- [ ] 下一次合批把四个完整 FQN 加入 required 集，提高普通测试下限；原有 selectors、nullable floors 与其他全部验证保留。
- [ ] Jenkins 实际编译和执行新增 4 项及保留回归，提供相同源码、配置、锁文件、features、toolchain、环境的完整前后输入封印。
- [ ] 按 [原计划](../../../optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor.md) 保留 Runtime63 / 08c / Runtime170 的分配、工作量、采样、规模及 soak 门槛；尚无本轮 writer P95 或 clone counter，不能以静态检查或 channel sampling 代替。
- [ ] 整批原有功能、资源和外部 Runtime03 绝对 P95 门槛全部通过后，再关闭相应验收项。

未运行 native/Cargo/Python unittest/perf。root 无自有构建、夹具或临时 helper，释放 0 B；其他会话及 UNKNOWN 产物/进程继续保留。
