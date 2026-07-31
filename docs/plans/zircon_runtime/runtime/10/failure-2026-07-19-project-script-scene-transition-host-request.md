---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: project-script-scene-transition-host-request
origin_plan: docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/woc/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - examples/woc/zircon-project.toml
  - examples/woc/assets/scenes/bootstrap.scene.toml
  - examples/woc/assets/scenes/eastbrook_mvp.scene.toml
  - examples/woc/scripts/woc_game/src/main.zr
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/host_requests.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
tests:
  - cargo test -p zircon_runtime project_script_scene_transition --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_app runtime_project_scene_transition --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime 10：项目脚本无法请求原子场景切换

## 来源执行者

- 来源计划：`docs/plans/woc/01-woc-zrvm-one-to-one-replication.md`
- 来源执行切片：M8 Eastbrook Vale desktop offline MVP product-shell wiring
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：最低共享原因是 dynamic project session 只在创建时装入 `default_scene`，没有项目脚本或 retained UI 可调用的场景生命周期请求；它属于 Runtime 10 session/host-request 边界，不是 WOC 场景内容缺陷。

## 失败现象与复现证据

WOC 的真实产品顺序是模式/角色选择、post-selection Welcome、Loading、Eastbrook 世界。当前项目必须把 `bootstrap.scene.toml` 保持为默认场景，否则直接把 `eastbrook_mvp.scene.toml` 设为 `default_scene` 会绕过选择与 Welcome 流程。

- `RuntimePreparedProject::load_default_scene` 只读取 manifest 的 `default_scene` 并在 session construction 期间装入一次。
- dynamic session 的 scene asset reload queue 只处理已装入场景资源的编辑/热重载，不提供从一个项目场景切换到另一个项目场景的产品生命周期。
- `zr.zircon.gameplay` host 只有实体、组件、输入、动画、导航和战斗操作，没有 `load_scene`、`change_scene` 或等价请求。
- `ZrRuntimeHostRequestV1` 当前只承载 IME、gamepad rumble 和 cursor 请求；没有项目场景 URI、切换策略、完成或失败结果。
- Vampire 示例不需要切换场景：它在同一个 world 中用 `gameplay.menu_state` 暂停后继续，因此不能证明 WOC 的 bootstrap -> Eastbrook 产品流。

这不是 Cargo 队列或 WOC 编译失败，也没有产品通过声明。WOC 可以继续完成纯 Shell 与场景内容，但不能在项目范围内可靠接通真实场景切换。

## 最低共享层根因

项目加载、scene world 所有权、脚本生命周期和 ABI host request 已分别存在，但 dynamic session 没有把它们组合成可由项目请求、在帧边界原子提交的场景转换事务。让每个游戏轮询 magic dynamic component、重启整个 runtime session 或冒用 asset hot reload 会复制生命周期政策，并无法定义旧脚本停机、输入/UI 捕获释放、失败回滚和完成通知。

## 架构修复验收

- 在 Runtime 10 拥有的中立契约中定义项目场景转换请求；脚本和 live project UI action 都能请求 project-root 内的 canonical `res://` scene URI，不暴露任意宿主文件路径。
- dynamic session 在确定的帧边界消费请求：停止旧 scene 的 script/UI 生命周期，准备并校验新 scene，在成功后一次性替换 active world，再启动新 scene 生命周期；同帧重复、并发和 supersede 规则必须显式。
- 加载、校验或启动失败保留旧 active scene，并通过稳定的项目可读结果和 diagnostics 报告错误；不得留下半替换 world、悬挂 focus/capture 或已销毁脚本绑定。
- scene URI、请求 id、策略与结果以版本化 ABI-safe/serde payload 穿过 dynamic boundary；desktop、mobile、browser 和 headless profile 对支持或拒绝策略有明确测试。
- 产品 fixture 从一个含交互 UI 的 bootstrap scene 切换到 gameplay scene，再切回或进入第二 scene；验证脚本 start/stop 次数、输入/UI 所有权、render extract、accessibility、失败回滚和 runtime teardown。
- WOC 只消费通用契约以完成 Welcome -> Loading -> Eastbrook，不在引擎加入 WOC URI、component id 或状态分支。

## 禁止临时方案

- 不得把 Eastbrook 直接设为默认场景并删除/绕过 Welcome 与离线 picker 验收。
- 不得让 WOC 写 magic dynamic component，再在 `dynamic_api` 硬编码轮询该 id 或 scene URI。
- 不得把 scene asset hot reload、runtime session 重启、进程重启或隐藏第二窗口冒充原子场景切换。
- 不得允许项目脚本提交绝对路径、越过 project root，或在失败时静默落到 bootstrap/default scene。

## 修复结果与回传

Open state: `待修复`; no pass is claimed. WOC 保持 bootstrap 为默认场景，继续完成可独立验证的 Shell、偏好、输入与 Eastbrook 内容切片，真实产品转换门保持开放。
