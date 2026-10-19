# 历史计划与数据归档索引

本页是历史资料的便携阅读入口。当前验证规范在 [验证政策](milestone-validation-policy.md)、[基础验收门槛](minimum-viable-engine-foundation.md)和 [MVP 验收索引](mvp-acceptance-index.md)。归档保留原始证据及其状态，不将历史静态审阅、截图或失败回执升级为当前接受。

## 已核验的归档批次

| 项目 | 2026-10-13 快照 |
| --- | ---: |
| 批次 | pasted-audit-fixes-01a13029 |
| docs/plans | 7,170 文件 / 133,569,215 逻辑字节 |
| docs/_data | 18,710 文件 / 1,377,018,903 逻辑字节 |
| 合计 | 25,880 文件 / 1,510,588,118 逻辑字节 |
| 本阶段从源码树释放 | 0 字节 |

用户选定的本机归档根为：

```
E:/cargo-targets/zircon-local/archive/zirconengine-docs-history/pasted-audit-fixes-01a13029/
```

根目录下的 docs/plans 和 docs/_data 保持原相对路径。四个曾发生并行修改的计划另有 versions/<old-sha12>/docs/plans/... 旧版本，不能用同步覆盖删除。清单位于本任务 reports/pasted-audit-fixes-01a13029/docs-migration/：

- archive-current-index-02.jsonl，SHA-256 `871b720433292ecd73e56d1f5e59eae43c8b7c52ef04fb99ab43cc102ad687a1`；
- archive-current-index-02-summary.json，SHA-256 `63b358351fabf781c6b303d42f4b45009e78d48491576d9ca085218f0018b9f1`。

索引02组合逐项复制记录、完整树比较、四项双版本协调和六项新增计划的独立源/目标哈希证据，并核对归档实际路径集合与长度。它证明该批次快照，未把后来继续编辑的 checkout 当成冻结来源。

<a id="archive-readback"></a>
## 按原路径读取与恢复

1. 阅读入口或迁入引用旁给出的原仓库相对路径，例如 `docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`。
2. 由归档维护方提供可读的实际归档根，使用“归档根 / 原相对路径”读取对应文件；保留原链接的相对目录关系。开发机当前可使用上述 E 盘位置，CI 不得假定存在该盘符。
3. 读取索引02对应行，核对路径、长度、SHA-256 及其逐项证据来源。已知双版本按索引选择标准路径或 versions 中的具体版本，不覆盖其中任何版本。
4. 需要恢复时先记录精确路径、哈希、owner 和目标；经授权的 Jenkins 归档生命周期操作恢复到明确目的地，并读回比较。不得镜像删除未知数据或构造替代源码树。
5. 旧脚本、协调器命令、job/lease/receipt 只解释历史，不恢复退役服务或重放队列；执行仍遵循 [退役规则](../tooling/coordinator-retirement.md)。

共享归档位置、维护身份、CI 认证/读回、备份与长期保留尚未被此本机快照证明。当前 checkout 的 consumers 仍有固定 docs/plans 和 docs/_data 读取；仅交付本入口不能声称它们已全部迁移，也不能删除原树。

## 数据含义与保留边界

- `docs/plans` 保存历史设计、阶段门槛、失败、审计与验证状态；迁出的当前政策和 MVP 门槛继续在本目录维护。
- `docs/_data/layout` 包含 `.zui` 源镜像、设计投影、字体/图片依赖、catalog、PNG 与逐项审阅证据。镜像、sourcePath、依赖和渲染指纹必须保留；prepared、visualStatus passed 和 review accepted 是不同事实，历史 PNG 不能证明当前源已通过。
- `docs/_data/source-comment-audit` 保存审计清单与统计来源；未因属于数据目录而被推断为可丢弃的缓存。
- 未知 owner、活动源、Git 元数据、共享依赖和其他聊天的产物均保留。用户 Full Access 只改变实际执行权限，不替代来源/哈希/消费者验证。

## 捕获后活动内容

快照生成时另有两项活动源未纳入该批次：

- `docs/plans/astra/features/editor/2026-10-13-editor07-play-hierarchy-completion-list.md`
- `docs/plans/optimize/zircon_editor/07/2026-10-13-play-hierarchy-sparse-capacity-and-selection-record.md`

之后仍可能有新增或修改。它们不能依据旧快照直接移除。当前原 plans/_data 均保留；消费者迁移、开发机与 CI 真实读回、逐文件当前源协调及源移除仍开放。释放空间必须按实际完成的移除与物理计量报告。
