# Zircon Runtime 独立 3D 游戏能力与 Vampire 示例计划

## Summary

当前 `zircon_runtime` 已有 3D 渲染、资产项目、场景/ECS、输入、动画/导航插件骨架和 ZrVM 脚本插件，但还不能算完整独立游戏运行时：预览入口还不加载项目主场景，帧循环未驱动 `LevelSystem` gameplay tick，ZrVM 只有包级生命周期，导航/动画/ZrVM 也未按项目清单稳定接入。

本计划按“引擎能力优先”推进：先把项目启动、场景 tick、ZrVM 实体脚本、导航烘焙/寻路打通，再在 `E:\Git\ZirconEngine\examples\vampire` 放一个可运行的第三人称吸血鬼肉鸽切片。

## Key Changes

- Runtime project launch: 给 `zircon_app` 的 runtime 入口增加 `--project <path>`，读取 `zircon-project.toml` 的 `default_scene`，用 `ProjectManager` 扫描资产并加载主场景；每帧调用 `LevelSystem::tick`，渲染提取继续来自当前关卡。
- Plugin wiring: 在 first-party runtime catalog 增加 ZrVM language provider 选择，项目清单启用 `rendering`、`texture`、`animation`、`navigation`、`zr_vm_language`；ZrVM 真后端作为显式 feature 启用，缺少本地 ZrVM native lib 时启动报告失败而不是静默降级。
- Script API: 扩展项目清单脚本段：
  - `[scripts] package_roots = ["scripts"]`
  - `startup_packages = ["vampire_game"]`
  场景实体增加 `script_bindings`，字段固定为 `package/module/enabled/properties`，生命周期为 `onStart(entity)`、`onUpdate(entity, dt)`、`onFixedUpdate(entity, dt)`、`onDestroy(entity)`。
- ZrVM host modules: 增加游戏脚本可用宿主能力：时间、日志、输入按键、实体/Transform 读写、动态组件 JSON 读写、查询 tag/component、spawn/despawn、导航 `findPath/nextPoint`、动画状态切换、简单随机数。
- Navigation: 使用现有 navigation runtime/native 做 v1 烘焙链路，场景里的 `NavMeshSurface` 生成 `.znavmesh` 资产；敌人脚本按固定间隔重新寻路到玩家，沿路径移动，失败时使用直线追踪降级并记录诊断。
- `E:\Git\ZirconEngine\examples\vampire`: 新建资产项目目录，包含 `zircon-project.toml`、主场景、ZrVM 脚本包、数值表、导航烘焙资产、CC0 资源许可证说明。默认资源从 [Kenney](https://kenney.nl/support)、[Quaternius](https://quaternius.com)、[Poly Haven](https://polyhaven.com/license) 选小体积 CC0/公开领域资源；不默认接入 Mixamo。

## Vampire Slice

- Controls: WASD 平面移动；第三人称跟随相机，固定偏移 `(0, 6, -8)`，看向玩家上方；攻击自动选择最近敌人。
- Player stats: HP 100，move speed 5.5 m/s，pickup radius 2.5 m，hit invulnerability 0.35s。
- Weapon v1: Blood Bolt，range 7.5m，cooldown 0.85s，damage 12，projectile speed 14m/s，pierce 1。
- Enemy v1: Ghoul，HP 18，move speed 3.2 m/s，contact damage 8，attack cooldown 0.75s，repath interval 0.4s。
- Progression: XP gem 每敌 1 点；升级需求 `8 + level * 4 + floor(level^1.35 * 3)`；升级池为 +20% damage、-15% cooldown、+10% move speed、+1 pierce、+25 max HP。
- Spawning: 初始每 2.0s 生成 1 个敌人，10 分钟线性压到 0.45s；最大存活 80；玩家死亡或 10 分钟生存达成即显示结果诊断/日志。

## Test Plan

- Runtime unit tests: project args parsing、项目清单脚本段解析、插件选择补全、默认场景加载、`LevelSystem` 每帧 tick。
- Script tests: ZrVM project package discovery、真实后端 feature 下调用 `onStart/onUpdate`、host input/transform/component/nav API 往返、热重载保留脚本状态。
- Navigation tests: 从示例场景烘焙 navmesh、敌人从多个出生点能寻路到玩家、阻塞/无路径时诊断明确且不崩溃。
- Example simulation: headless 跑 120 秒，断言玩家移动、自动攻击、敌人扣血/死亡、XP 升级、敌人沿 navmesh 接近玩家。
- Manual acceptance: 运行 `cargo run -p zircon_app --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,first-party-zr-vm-real-backend" --bin zircon_runtime -- --project E:\Git\ZirconEngine\examples\vampire`，能看到第三人称角色、WASD 移动、敌人寻路、自动攻击和基础数值循环。

## Assumptions

- “完整脚本语言”第一版锁定为现有 ZrVM，而不是 Lua/JS；目标是补齐游戏实体脚本生命周期和宿主 API。
- `examples/vampire` 是资产项目目录，不加入 Cargo workspace。
- v1 不做多人、存档、复杂碰撞、完整编辑器 UI、商店级美术表现；动画若受当前 glTF 动画导入限制影响，先保证 gameplay 正确，动画以可诊断 fallback 运行。
- 入库外部资源只选体积可控、许可清晰的 CC0/免费资源，并保留来源和许可证文件。