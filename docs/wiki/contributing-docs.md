---
related_code:
  - docs/wiki/index.md
  - .codex/skills/zircon-project-skills/code-module-docs-maintenance/required-doc-header-format.md
  - .codex/skills/zircon-project-skills/code-module-docs-maintenance/write-module-docs/SKILL.md
implementation_files:
  - docs/wiki
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - .codex/skills/zircon-project-skills/code-module-docs-maintenance/SKILL.md
tests:
  - tools/check_conventions.py
  - docs/plans/mvp/index.md
doc_type: testing-guide
---

# Wiki 文档维护

`docs/wiki` 是面向使用者和 Rust 开发者的持久化说明层；源码、聚焦测试和 owner 计划仍是当前事实的第一来源。页面应帮助读者做正确调用，而不是复制实现细节或提交日志。

## 页面类型

| `doc_type` | 用途 |
| --- | --- |
| `category-index` | 分区导航、读者路径和状态矩阵 |
| `module-detail` | 一个 crate/module 的稳定职责、边界和 API |
| `workflow-detail` | 启动、导入、编辑、保存、导出等操作流程 |
| `testing-guide` | 可重复验证、平台限制和故障排查 |
| `milestone-detail` | 与计划退出条件绑定的状态说明 |

## Frontmatter

所有代码相关页面在第一行写 YAML frontmatter，`related_code` 必须排第一：

```yaml
---
related_code:
  - zircon_runtime/src/core/runtime/runtime.rs
implementation_files:
  - zircon_runtime/src/core/runtime/runtime.rs
plan_sources:
  - user: 2026-09-09 <request summary>
tests:
  - zircon_runtime/src/core/runtime/tests
doc_type: module-detail
---
```

路径必须是仓库相对路径。若代码与文档分开提交，路径可以暂时尚未出现在当前 checkout；`tools/wiki_site.py validate` 会把这类元数据目标报告为 warning，并在源码快照齐全时用 `--strict-metadata` 将其升级为错误。描述多个 owner 时列出所有被解释的代码，不要只列最近编辑的文件。

## 写作准则

- 先写意图和边界，再写机制和示例。
- 只记录可持续的事实、公共契约、错误语义和未来修改必须遵守的不变量。
- 用状态标签区分源码、实验和规划；必要时链接 owner 计划。
- 示例以 crate root re-export 为首选，展示 `Result`、feature 和资源/句柄释放。
- 不在 Wiki 中复制完整源文件、每次提交变更或临时验证日志。
- 发现旧事实错误时直接修正，不追加“修订历史”段落。

## 网站导入

`navigation.yaml` 提供稳定的 slug、分区、标题和前置阅读关系。网站生成器可以把 Markdown frontmatter 作为页面元数据，把 `navigation.yaml` 作为侧边栏/搜索索引的初始输入。生成器应保留相对链接，并对缺失的站内页面发出错误；代码、测试和计划路径的暂时缺失由元数据 warning 标出。

## 提交前检查

```powershell
rg --files docs/wiki
rg -n '^---$|^related_code:|^doc_type:' docs/wiki
python tools/check_conventions.py --help
```

再用脚本检查 frontmatter 引用的路径、内部 Markdown 链接和标题重复。文档任务只新增/修改目标页面，不覆盖其他 Session 的工作区改动。
