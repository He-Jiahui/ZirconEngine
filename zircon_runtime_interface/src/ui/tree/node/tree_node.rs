use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiNodePath, UiStateFlags};
use crate::ui::focus::UiFocusContract;
use crate::ui::layout::{
    Anchor, BoxConstraints, LayoutBoundary, Pivot, Position, UiContainerKind, UiFrame, UiMargin,
    UiScrollState,
};
use crate::ui::navigation::UiNavigationContract;

use super::{
    UiCursor, UiDirtyFlags, UiInputPolicy, UiLayoutCache, UiPointerEvents, UiTemplateNodeMetadata,
    UiVisibility,
};

/// 保留式树中的节点数据；拓扑、作者契约、交互状态和阶段缓存分别保存。
/// 字段和构建方法不执行整图或数值准入；调用方须维护父子关系一致且无环。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTreeNode {
    /// 树内节点标识；作为映射键使用时由调用方保持两者一致。
    pub node_id: UiNodeId,
    /// 节点路径数据；不由本结构检查唯一性或与父级拼接。
    pub node_path: UiNodePath,
    /// 父节点与子节点列表须由树修改方配套维护。
    pub parent: Option<UiNodeId>,
    /// 有序子节点标识；与父子边的布局槽分别存储。
    pub children: Vec<UiNodeId>,
    /// 兼容交互状态；旧可见位会参与有效可见性，旧脏位仅折算命中、绘制和输入域。
    pub state_flags: UiStateFlags,
    /// 本节点的布局、绘制与命中可见性；祖先限制由 Runtime 另行合并。
    #[serde(default)]
    pub visibility: UiVisibility,
    /// 作者尺寸约束；赋值和反序列化本身不进行数值验证。
    #[serde(default)]
    pub constraints: BoxConstraints,
    /// 节点定位的锚点。
    #[serde(default)]
    pub anchor: Anchor,
    /// 节点定位的枢轴。
    #[serde(default)]
    pub pivot: Pivot,
    /// 节点定位偏移。
    #[serde(default)]
    pub position: Position,
    /// 容器种类；赋值不会同步滚动状态或裁剪开关。
    #[serde(default)]
    pub container: UiContainerKind,
    /// Node-owned content inset from the authored layout contract.
    ///
    /// This is distinct from `UiSlot::padding`: slot padding belongs to one
    /// parent/child edge, while node padding insets every child of this node
    /// and contributes to intrinsic measurement.
    #[serde(default)]
    pub layout_padding: UiMargin,
    /// Preserves an explicitly-authored stretch axis in linear layout instead of treating
    /// a default-looking stretch constraint as a content-driven fixed fallback.
    #[serde(default)]
    pub layout_stretch_width: bool,
    /// 显式作者拉伸的高度轴标记，与宽度轴分别保留。
    #[serde(default)]
    pub layout_stretch_height: bool,
    /// 可选滚动状态；是否与容器类型配套由构建方保证。
    #[serde(default)]
    pub scroll_state: Option<UiScrollState>,
    /// 本节点焦点契约；焦点路由还会使用树级条件。
    #[serde(default)]
    pub focus: UiFocusContract,
    /// 本节点导航契约。
    #[serde(default)]
    pub navigation: UiNavigationContract,
    /// 本节点的输入声明；Runtime 从本节点沿父链查找最近显式值。
    pub input_policy: UiInputPolicy,
    /// 与可见性共同限制自身及子树的指针命中。
    #[serde(default)]
    pub pointer_events: UiPointerEvents,
    /// `None` allows cursor resolution to continue through the hit path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<UiCursor>,
    /// 命中裁剪链使用的节点开关；具体裁剪几何来自布局缓存。
    pub clip_to_bounds: bool,
    /// Runtime 向祖先传播布局失效时使用的边界。
    pub layout_boundary: LayoutBoundary,
    /// 绘制排序主键；同层再按绘制序号和节点标识排序。
    pub z_index: i32,
    /// 节点映射分配的绘制序号，也被树内实例标识查询使用。
    pub paint_order: u64,
    /// 待消费的各失效域；获得可变借用只登记候选，不会自动设置这些位。
    pub dirty: UiDirtyFlags,
    /// 测量和排列共享的阶段缓存。
    pub layout_cache: UiLayoutCache,
    /// 构建器保留的作者元数据；后续显式属性写入可更新它。
    #[serde(default)]
    pub template_metadata: Option<UiTemplateNodeMetadata>,
}

impl UiTreeNode {
    /// 用给定标识和路径创建尚未挂树的节点，默认可见、启用且没有交互能力。
    /// 返回值无父子边、滚动状态或元数据；加入树时再分配绘制序号。
    pub fn new(node_id: UiNodeId, node_path: UiNodePath) -> Self {
        Self {
            node_id,
            node_path,
            parent: None,
            children: Vec::new(),
            state_flags: UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            },
            visibility: UiVisibility::Visible,
            constraints: BoxConstraints::default(),
            anchor: Anchor::default(),
            pivot: Pivot::default(),
            position: Position::default(),
            container: UiContainerKind::default(),
            layout_padding: UiMargin::default(),
            layout_stretch_width: false,
            layout_stretch_height: false,
            scroll_state: None,
            focus: UiFocusContract::default(),
            navigation: UiNavigationContract::default(),
            input_policy: UiInputPolicy::Inherit,
            pointer_events: UiPointerEvents::Auto,
            cursor: None,
            clip_to_bounds: false,
            layout_boundary: LayoutBoundary::ContentDriven,
            z_index: 0,
            paint_order: 0,
            dirty: UiDirtyFlags::default(),
            layout_cache: UiLayoutCache::default(),
            template_metadata: None,
        }
    }

    /// 消费并返回设置后的节点；本组构建方法只赋值，不通知树索引或传播失效。
    /// 此方法仅预置节点框，不设置测量有效位。
    pub fn with_frame(mut self, frame: UiFrame) -> Self {
        self.layout_cache.frame = frame;
        self
    }

    pub fn with_state_flags(mut self, state_flags: UiStateFlags) -> Self {
        self.state_flags = state_flags;
        self
    }

    pub fn with_visibility(mut self, visibility: UiVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// 合并本节点的显式可见性与旧可见位；不遍历祖先。
    pub fn effective_visibility(&self) -> UiVisibility {
        self.visibility.effective(self.state_flags.visible)
    }

    pub fn is_render_visible(&self) -> bool {
        self.effective_visibility().is_render_visible()
    }

    pub fn is_self_hit_test_visible(&self) -> bool {
        self.effective_visibility().allows_self_hit_test()
    }

    pub fn allows_child_hit_test(&self) -> bool {
        self.effective_visibility().allows_child_hit_test()
    }

    /// 本地可见性与指针策略共同准入；祖先策略、裁剪和路由仍由 Runtime 检查。
    pub fn allows_self_pointer_hit_test(&self) -> bool {
        self.is_self_hit_test_visible() && self.pointer_events.allows_self_hit_test()
    }

    pub fn allows_child_pointer_hit_test(&self) -> bool {
        self.allows_child_hit_test() && self.pointer_events.allows_child_hit_test()
    }

    /// 仅查询声明的穿透值，不执行事件传播；当前消费缺口见指针策略定义处的旧问题标记。
    pub fn is_pointer_passthrough(&self) -> bool {
        self.pointer_events.is_passthrough()
    }

    /// 本地启用、可聚焦及绘制可见的候选判定，不完成树级焦点选择。
    pub fn is_focus_candidate(&self) -> bool {
        self.state_flags.enabled
            && (self.state_flags.focusable || self.focus.focusable)
            && self.is_render_visible()
    }

    /// 本地指针能力候选；继承输入策略和祖先路径需由命中、路由阶段另行检查。
    pub fn supports_pointer(&self) -> bool {
        self.state_flags.enabled
            && self.allows_self_pointer_hit_test()
            && (self.state_flags.clickable
                || self.state_flags.hoverable
                || self.state_flags.focusable
                || self.focus.focusable)
    }

    /// 原样保存尺寸约束；此处不调用解析、求解或有限值检查。
    pub fn with_constraints(mut self, constraints: BoxConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_pivot(mut self, pivot: Pivot) -> Self {
        self.pivot = pivot;
        self
    }

    pub fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// 只保存容器类型；需要滚动状态和裁剪时由构建方分别设置。
    pub fn with_container(mut self, container: UiContainerKind) -> Self {
        self.container = container;
        self
    }

    pub fn with_layout_padding(mut self, layout_padding: UiMargin) -> Self {
        self.layout_padding = layout_padding;
        self
    }

    pub fn with_layout_stretch_axes(mut self, width: bool, height: bool) -> Self {
        self.layout_stretch_width = width;
        self.layout_stretch_height = height;
        self
    }

    /// 包装为有值的滚动状态；不检查容器类型，也不立即改变排列几何。
    pub fn with_scroll_state(mut self, scroll_state: UiScrollState) -> Self {
        self.scroll_state = Some(scroll_state);
        self
    }

    pub fn with_focus_contract(mut self, focus: UiFocusContract) -> Self {
        self.focus = focus;
        self
    }

    pub fn with_navigation_contract(mut self, navigation: UiNavigationContract) -> Self {
        self.navigation = navigation;
        self
    }

    pub fn with_input_policy(mut self, input_policy: UiInputPolicy) -> Self {
        self.input_policy = input_policy;
        self
    }

    pub fn with_pointer_events(mut self, pointer_events: UiPointerEvents) -> Self {
        self.pointer_events = pointer_events;
        self
    }

    /// 保存光标声明；不在此处向宿主请求切换光标。
    pub fn with_cursor(mut self, cursor: UiCursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn with_layout_boundary(mut self, layout_boundary: LayoutBoundary) -> Self {
        self.layout_boundary = layout_boundary;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    pub fn with_clip_to_bounds(mut self, clip_to_bounds: bool) -> Self {
        self.clip_to_bounds = clip_to_bounds;
        self
    }

    pub fn with_template_metadata(mut self, template_metadata: UiTemplateNodeMetadata) -> Self {
        self.template_metadata = Some(template_metadata);
        self
    }
}
