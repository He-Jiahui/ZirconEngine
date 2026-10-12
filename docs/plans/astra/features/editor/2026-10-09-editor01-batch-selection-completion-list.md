# Editor01 · 批量验证选择完成列表

日期：2026-10-09。对应[优化记录](../../../optimize/zircon_editor/01/2026-10-09-batch-ui-namespace-selection.md)。

## 已完成任务清单修复

- [x] 独审定位九个既有 binding 用例因模块 namespace 顺序错误而漏选。
- [x] v4 修正一个筛选和九个完整名称，重新核对实际声明及定义。
- [x] 保留全部原用例/断言、最低 132，以及其他十四项命令和原 flags/features/profile/target。
- [x] 源码确认 76 + 47 + 1 + 8 = 132 个活跃非 ignored UI 用例无交集；保留 Runtime 10 个追加选择和 Release 两个追加 ignored producers。

实际清单 SHA-256：`5c9affb701a78b47b5b4d81f9059d8bcc497b77cabe358ad79776ade4cc19fa6`。源码用例清单与真实 native 运行数量分开记录。

## 待实际验收

- [ ] 后续源码接入后重新绑定全部来源、toolchain 与原性能 producers，批量提交一次。
- [ ] 从最终 receipt 核完整实际 unique names、非零 case 数、全部原断言及 Release marker/矩阵。
- [ ] 原 139 事件、96 强静态可达性、产品 F0、性能和全 workspace 门通过。

当前未声称 native、Jenkins 或性能接受。
