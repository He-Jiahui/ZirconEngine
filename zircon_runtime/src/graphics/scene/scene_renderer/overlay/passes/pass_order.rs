// TODO: [CR-SCENE-UI-0001] 顺序测试只比较此常量，未验证实际记录顺序；需采集 pass 记录或图执行序列，覆盖实际调用重排。
#[cfg(test)]
pub(crate) const PASS_ORDER: &[&str] = &[
    "PreviewSkyPass",
    "BaseScenePass",
    "SelectionOutlinePass",
    "WireframePass",
    "GridPass",
    "SceneGizmoPass",
    "HandlePass",
];
