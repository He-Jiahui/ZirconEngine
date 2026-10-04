use serde::{Deserialize, Serialize};

/// 网格资产与绘制端共享的图元组装约定；法线/切线生成只接受支持的拓扑。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMeshTopology {
    #[default]
    TriangleList,
    TriangleStrip,
    LineList,
    LineStrip,
    PointList,
}
