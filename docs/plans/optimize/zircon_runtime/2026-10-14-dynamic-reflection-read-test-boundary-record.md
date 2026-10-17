# Runtime63：反射读取结构回归对齐实际共享 helper

记录时间：2026-10-15T07:29:33.455785+00:00。核心实现：camera_mesh_core / gpt-6.1-sol max；独立复核：batch12_evidence / gpt-6-luna max。

现有 dynamic_component_reflection_read_helpers_use_direct_success_branches 测试截取 read_field 入口 wrapper，却要求其中包含已委托给 read_declared_field 的转换分支。源码静态核对确认原16个谓词有2个绑定了错误范围；这次修复现有测试，不修改生产读取行为。

成功分支、UnsupportedConversion 和禁止 closure 的断言绑定实际 read_declared_field；按名和槽位入口分别检查字段声明解析及共享 helper 委托，防止仅验证不可达 helper。JSON presence helper 的截取终点收紧到 declared_field_by_slot，原组件缺失、未知字段、对象/字段存在性及适配语义保留。

唯一既有测试文件在原27030B raw before 上修改；原9行注释及另外15个 ordinary body 保留。16个 ordinary/0ignored，source-new=0，FQN不变；不会把修复测试计为新增测试。

原 B16 的16个 Cargo commands 对该 leaf 有效选择为0/16，合批时追加16个既有精确 FQN。因此 Runtime floor121+Animation5+watch19+existingReflection16=161，required64+5+19+16=104；原 argv、required名字、per-prefix包含null、其他14条命令、2Python、16性能producer和legacy11门完全保留。

root scoped change guard 与独立 source/packet peer 实际通过；4个只读生产/wiring witness 与1个Runtime63计划 pin核对当前字节。源码、静态、native分别记录；静态谓词满足不构成原生测试通过。

本修复归属 Runtime63 RSR-G24/G25、G36、G48 的读取验证维护；未发现专属分配/P95门。CR-ZCA-R010-0001、G47/G48及原 Runtime63总体验收保留OPEN，待同源 Jenkins 原生及全部原性能/resource/Runtime03 AND回执。

核心和根会话未运行native或编译；批量交接后继续 Runtime/Editor源功能修复，不轮询编译状态。

[原计划](63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor.md)；[补丁](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime63-reflection-read-test-boundary.patch)；[源码证据与完整 FQN](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime63-reflection-read-test-boundary-evidence.json)；[静态检查](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime63-reflection-read-test-boundary-static.json)；[Astra 完成列表](../../astra/features/runtime/2026-10-14-runtime63-dynamic-reflection-read-test-boundary-completion-list.md)；[冻结 Runtime03 绝对 P95 标准](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-absolute-p95-criteria.json)。

产物：仅预登记补丁、元数据、静态报告和计划记录；未创建编译/cache/临时 helper/native fixture/源码副本。实际释放0B，必要验收和失败证据保留。历史 UNKNOWN 进程/回执不因本记录而关闭。
