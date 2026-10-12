# Editor01 · 批量验证的九项漏选修复

日期：2026-10-09。对应[原计划](../01-retained-ui-architecture-performance-review.md)。这是任务验证清单修复，未修改生产代码、既有测试体或工具实现。

## 根因与修正

Rust 的实际声明顺序为 `builtin::workbench_window_template_bindings::tests`。此前任务清单误写为 `builtin::tests::workbench_window_template_bindings`，使九个既有 Assets/binding 用例全部漏选。

已在 `commands-draft-v4.json` 修正 UI #15 的一个 OR filter 和九个 required 完整名称，其他十四项命令及所有 flags/features/profile/target 保留。原最低 132 不降低，九个原测试及断言保留。原 15 项批量配置继续承载 Runtime、Editor、App 和原 Release producers。

## 已核对证据

独审沿 70 个选定文件及声明祖先核对用例，采用检查合计 85 个 scoped 文件。UI 的 132 个真实活跃、非 ignored 用例分组为原 76 + caller 47 + Details 新增 1 + Module 新增 8，无交集。旧 v3 筛选实际只覆盖 123 个；纠正 namespace 后应覆盖 132，实际 native 尚未运行。

- 独审 `batch-native-v4-independent-case-review-v1/review.json` SHA-256：`c9c319d7a15105492cc8b4dc285872c8012d017a79a2ccc3b13042659d20e8a3`。
- inventory SHA-256：`48e5d11cb9bbf72ce4ae27ff5a64f7a2c5cff5a48d7efcc7a35e06dbb74cb735`。
- 精确修复清单 SHA-256：`b69b1e9403b64d46c072536e3038ebb9514245292cfeba9f6d39c9e742142680`。
- 实际任务清单 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v4/commands-draft-v4.json` SHA-256：`5c9affb701a78b47b5b4d81f9059d8bcc497b77cabe358ad79776ade4cc19fa6`。两个声明和九个 fn 定义重新核对，JSON 只改变一个 Cargo argument 及九个要求名称。

同一复核确认 Runtime #2 新增选择并集为 10：三项新源码用例与七项旧回归；与原七 filters 不重叠。Release #9 的两个追加 producers 均真实 ignored，原八项要求保留。54 行矩阵和 20 原始样本分别核对，不作为 native case 数量。

## 待完成

v4 仍是未 source-bound、未提交或启动的草案。Particle、六菜单及其他实际接入后需刷新精确名称与来源。原 139 项逐事件断言、96 强静态可达性、产品 F0、全 workspace 和原性能预算保持开放。测试名称/数量必须从最终 request 的实际 receipt 核对。

详见[Editor 完成列表](../../../astra/features/editor/2026-10-09-editor01-batch-selection-completion-list.md)。
