---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/core/math/mod.rs
implementation_files:
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
plan_sources:
  - user: 2026-09-09 场景、资产与资源运行时 Wiki
tests:
  - zircon_runtime/src/scene/tests
  - zircon_runtime/src/asset/tests
doc_type: category-index
---

# 场景与资产

本分区说明 ZirconEngine 从项目资产到运行时场景的完整链路。场景由 `World`（ECS 存储）承载，`LevelSystem` 负责世界生命周期和时钟；资源以 `ResourceId`/`ResourceHandle<T>` 标识，由资产管线导入并由资源注册表管理。编辑器、脚本和渲染层通过反射和稳定路径访问同一份数据。

## 阅读路径

| 主题 | 页面 |
| --- | --- |
| 场景生命周期、Level 和世界替换 | [scene-runtime.md](scene-runtime.md) |
| ECS 实体、层级、组件和查询 | [ecs-world.md](ecs-world.md) |
| 内置场景组件（变换、激活、渲染、光照、物理） | [scene-components.md](scene-components.md) |
| 动态场景捕获、预检、补丁与会话归档 | [dynamic-scenes.md](dynamic-scenes.md) |
| 项目清单、资产导入和热重载 | [project-assets.md](project-assets.md) |
| 资源句柄、注册表、就绪状态 | [resource-runtime.md](resource-runtime.md) |
| 数学、坐标约定与变换验证 | [math-transforms.md](math-transforms.md) |

## 能力状态

上述页面以 2026-09 源码为准。标记“已实现”的 API 已在 `zircon_runtime` 导出；标记“按 feature”或“实验性”的功能需要对应 Cargo feature 或仍受测试覆盖范围约束。
