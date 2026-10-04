---
related_code:
  - .jenkins/deployment-spec.json
  - .jenkins/workflow-spec.json
  - tools/jenkins/
  - .jenkins/pipeline/
plan_sources:
  - docs/plans/jenkins-session-coordination/01-single-host-jenkins-coordination.md
tests:
  - "Source-bound support regression and authoritative Jenkins runtime receipts"
doc_type: milestone-output-record
---

# 已接受里程碑

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
| --- | --- | --- | --- | --- |
| M0 | 退役边界、Home／账号／进程／插件与迁移来源基线 | `accepted` | 2026-10-03 | `.jenkins/state/migration-baseline/m0-acceptance.json`；79 个目标包 SHA-256 核对 |
| M1 | 独立支持层、生命周期故障修复及请求转发身份保护的源码绑定回归；不替代最新部署运行验收 | `accepted-support-regression` | 2026-10-03 | `final-support-regression.json`：217 项覆盖，216 项通过、1 项环境跳过；完整发现加受影响模块复验，原始失败记录保留 |
| M3 | 有界 Git fixture 的纯注释五阶段，保留外来暂存，零编译和通知 | `accepted` | 2026-10-03 | `zircon-flow #7`；`m3-runtime-acceptance.json`；正式 acceptance `67c8fcb3679afca947f80db180782a91fe87b05ac2df76a8943c762d8caef9a4` |
| M4 | 两包 fixture 的模块、跨模块、原失败修复与负向测试；产物物理路径及 native 终止核对 | `accepted-bounded-fixture` | 2026-10-03 | `zircon-flow #10/#12/#14`，预期失败 `#15`；`jenkins-cargo-*.json` 中正式回执、Git 树和外来暂存核对 |

回执绑定各自的封存输入、覆盖及原 driver；最新驱动的运行复验和 M2、M5–M8 的剩余门槛见所属计划及机器证据索引。运行、缓存、私有配置和证据留在 Git 忽略的 `.jenkins/state`，不包含在本记录中。
