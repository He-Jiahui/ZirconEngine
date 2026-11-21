# 暂存清单应只将可暂存的 UI 缓存条目列作编译制品

[UI 缓存暂存器](../../../tools/build/zircon_build_asset_staging.py)只选择 `.zuiart` 和 `.zuicache`，忽略同目录中的编译摘要等辅助文件。[清单](../../../tools/build/zircon_build_staging_manifest.py)在 `assets/ui/compiled_artifacts` 下反查缓存来源时却不筛选后缀。Windows / Python 3.14.4 实测：开发 profile 下，暂存器复制 `widgets/panel.zuiart`，跳过 `widgets/compile-summary.json`；把这份辅助文件作为残留项放入产物目录再生成清单，清单将两者都登记为 `ui_compiled_artifact`。这是清单对不一致输入的错误接受，不是正常暂存过程会复制 JSON。

UE `DeploymentContext.StageFile` 将目标及实际选中的输入交给 `FilesToStage.Add`；`EpicGames.Core.Tests.FileSetTests` 分别核对完整集合与过滤子集。ZE 保留 Python 发布编排、Rust UI 制品格式和按 profile 裁剪的资源根；本项迁移“未被选中的输入不是有效来源”这一可观察行为，不引入 UAT，也不改变 Rust 运行时。与 #244 的 Runtime/Editor 根归属不同，这里是已选缓存根**内部的文件资格**。

验收：经真实暂存器和清单入口验证同目录下合法制品保留、非制品不被缓存认领；同路径由合法普通资产根交付时，仍可记录为 `source_asset`。不把 Windows 上的发布工具单测当作跨平台产品启动或 Rust 编译验收。
