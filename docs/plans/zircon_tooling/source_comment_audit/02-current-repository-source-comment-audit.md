# ZirconEngine 全仓源码注释审查：当前工作树第二轮

**目标：** 对当前第一方源码逐对象、逐语句块核对调用关系和设计意图，添加有解释价值的中文注释；不改变程序行为或公共 API。产品、插件、工具、示例、测试、着色器和可执行文本资源均在范围内。

**授权与边界：** 用户授权多个 `gpt-6-luna` / `max` subagents（最多 30 并发）、协调器精确快照阶段提交及其企业微信自动通知。保留既有改动、共享索引和外来租约。旧 `01` 计划与旧账本保持原状；第二轮以 [统一规范](../../../cli-and-tooling/source-comment-audit/round-02/README.md) 和独立账本记录当前审查。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-source-comment-audit-round-02",
  "goal": "Review current first-party source call chains and annotate verified intent without changing program behavior or public APIs",
  "milestones": [
    {"id": "M1", "title": "Current inventory, comment standard, and ownership", "depends_on": []},
    {"id": "M2", "title": "Public contracts, reflection, and plugin SDK", "depends_on": ["M1"]},
    {"id": "M3", "title": "Runtime kernel, manager, and neutral framework", "depends_on": ["M2"]},
    {"id": "M4", "title": "Runtime asset, scene, graphics, text, UI, and remaining systems", "depends_on": ["M3"]},
    {"id": "M5", "title": "Editor contracts, authoring, host, UI, and resources", "depends_on": ["M4"]},
    {"id": "M6", "title": "First-party plugin families and their tests", "depends_on": ["M5"]},
    {"id": "M7", "title": "Hub, app, hosts, tools, examples, and executable resources", "depends_on": ["M6"]},
    {"id": "M8", "title": "Final hashes, callers, issues, validation, and submission reconciliation", "depends_on": ["M7"]}
  ]
}
```

## 执行与验收

- 每批分配精确文件清单；主审取得路径租约后，一个 agent 负责一个文件的写入。调用方可跨仓只读追踪，相关测试随模块审查。
- 复用旧证据须核对当前内容与影响调用方；哈希相同本身不能证明调用上下文仍然成立。过期、缺失、未知后缀和外来占用均须明确记录，不能伪装为已完成。
- 每批验收包含调用证据与注释复核、仅本任务增量的 diff、语言解析或格式检查、问题标签及覆盖账本对应检查。协调器要求的实际票据未通过时保持待验收。
- 既有未提交源码可以只读审查，但不得整文件混入阶段提交。需要注释的位置先生成有前后哈希的增量建议，待合法归属与精确快照条件满足后处理。
- 首次提交前诊断现存索引锁。阶段提交及通知由协调器管理，不手工暂存、删除锁或绕过门槛。
- 全部当前第一方文件有与最终内容一致的记录、调用证据复核有效、待处理为零、源码 `BUG:` / `TODO:` 与问题账本一致、必要提交与验证回执终结后才能关闭 Goal。

## 阶段状态

阶段概述与受管回执写入 [第二轮子记录](02/2026-09-30-stage-status.md)。逐文件证据、问题及占用清单保存在账本中，不复制到计划正文。
