//! 从模板组件元数据解析作者布局度量，并补入受支持组件的内容期望尺寸；不负责节点排布或绘制。

use toml::Value;
use zircon_runtime_interface::ui::{
    design_tokens::EditorDesignTokens, layout::UiSize, tree::UiTemplateNodeMetadata,
};

#[derive(Default)]
/// 保存从节点属性解析的度量，供材料组件把已测内容换算为期望尺寸；Dropdown 的 caret 占位由独立路径补入。
///
/// # 使用与约束
/// 由 `resolve` 在单次测量中构造，不持有节点元数据的借用；`Default` 的十项度量均为零。
/// 属性解析只夹取下界，不保证有限值或设置上界；直接构造此结构也不会触发校验。
///
/// # 参见
/// `measure_material_content` 决定组件和图标准入，`apply_to_content` 只合成尺寸。
pub(super) struct MaterialLayoutMetrics {
    pub padding_left: f32,
    pub padding_right: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub spacing: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub icon_size: f32,
    pub leading_slot_width: f32,
    pub trailing_slot_width: f32,
}

impl MaterialLayoutMetrics {
    /// 从节点属性构造本次内容测量所用的作者度量。
    ///
    /// # 参数
    /// `metadata` 只在调用期间借用；只读取 `attributes`，不合并 `style_overrides`。
    ///
    /// # 返回与约束
    /// 没有任何已识别键时返回 `None`；键存在但值不可转换时仍返回 `Some`，对应值为零。
    /// 未指定项保持零；转换结果按零夹取下界，但不在这里拒绝正无穷或设置上界。
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Option<Self> {
        let mut authored_metric = false;
        let mut metrics = Self {
            padding_left: 0.0,
            padding_right: 0.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
            spacing: 0.0,
            min_width: 0.0,
            min_height: 0.0,
            icon_size: 0.0,
            leading_slot_width: 0.0,
            trailing_slot_width: 0.0,
        };
        for (key, value) in &metadata.attributes {
            let target = match key.as_str() {
                "layout_padding_left" => &mut metrics.padding_left,
                "layout_padding_right" => &mut metrics.padding_right,
                "layout_padding_top" => &mut metrics.padding_top,
                "layout_padding_bottom" => &mut metrics.padding_bottom,
                "layout_spacing" => &mut metrics.spacing,
                "layout_min_width" => &mut metrics.min_width,
                "layout_min_height" => &mut metrics.min_height,
                "layout_icon_size" => &mut metrics.icon_size,
                "layout_leading_slot_width" => &mut metrics.leading_slot_width,
                "layout_trailing_slot_width" => &mut metrics.trailing_slot_width,
                _ => continue,
            };
            authored_metric = true;
            *target = value_as_f32(value).unwrap_or(0.0).max(0.0);
        }
        authored_metric.then_some(metrics)
    }

    /// 把已测内容与组件装饰合成为期望尺寸。
    ///
    /// # 参数
    /// - `content` 是调用方已测得的内容尺寸，本方法不重新测量内容。
    /// - `has_icon` 由调用方依据组件与源属性决定；仅在此标志为真且内容宽度为正时加入图文间距。
    /// - `trailing_extra_width` 是额外尾部占位，Dropdown 用它传入 caret 一侧的空间需求。
    ///
    /// # 返回与约束
    /// 宽度合入图标、槽位、内边距与额外尾宽；高度取内容和启用图标的较大值再加上下内边距。
    /// 返回值最后应用最小宽高，不设置上界或保证有限性；只返回新 `UiSize`，不修改节点树。
    fn apply_to_content(
        &self,
        content: UiSize,
        has_icon: bool,
        trailing_extra_width: f32,
    ) -> UiSize {
        let icon_size = if has_icon { self.icon_size } else { 0.0 };
        let icon_text_spacing = if has_icon && content.width > 0.0 {
            self.spacing
        } else {
            0.0
        };
        let width = content.width
            + icon_size
            + icon_text_spacing
            + self.leading_slot_width
            + self.trailing_slot_width
            + self.padding_left
            + self.padding_right
            + trailing_extra_width;
        let height = content.height.max(icon_size) + self.padding_top + self.padding_bottom;
        UiSize::new(width.max(self.min_width), height.max(self.min_height))
    }
}

/// 在叶内容测量后补充组件皮肤声明的空间需求；无已知组件或无作者指标时返回 None。
/// 这里只影响期望尺寸，不选择绘制皮肤，也不从视觉内容反推作者未声明的外边距。
/// 例外：Dropdown 无 `layout_*` 指标时仍返回尺寸；caret 度量优先取样式覆盖或属性，缺失时使用设计令牌。
///
/// # 参数
/// - `metadata` 是本次测量借入的可选模板元数据；缺失时无法判断组件与作者度量。
/// - `content` 是上游已测得的内容尺寸；这里补入组件空间需求，不重新测量文本或子节点。
///
/// # 返回
/// 支持的组件具有已识别 `layout_*` 属性时返回合成尺寸；Dropdown 还允许使用零值度量继续计算装饰。
/// 缺失元数据、不支持的组件，或其他组件没有任何已识别度量时返回 `None`。
///
/// # 调用与生命周期
/// 容器内容和叶内容测量路径都会调用这里；结果仅是期望尺寸，不写回树，也不保留元数据借用。
/// 解析与合成不提供有限性或最大尺寸保证；排布和后续绘制由各自阶段负责。
///
/// # 参见
/// `MaterialLayoutMetrics` 保存此次解析结果；Dropdown 装饰采用 `dropdown_adornment_width` 的覆盖优先级。
pub(super) fn measure_material_content(
    metadata: Option<&UiTemplateNodeMetadata>,
    content: UiSize,
) -> Option<UiSize> {
    let metadata = metadata?;
    if !supports_material_layout(metadata.component.as_str()) {
        return None;
    }

    let is_dropdown = metadata.component == "Dropdown";
    let metrics = MaterialLayoutMetrics::resolve(metadata)
        .or_else(|| is_dropdown.then(MaterialLayoutMetrics::default))?;
    // Dropdown 走尾部 caret 宽度；其他组件由非空图标源属性或正尺寸 IconButton 启用图标。
    let has_icon = !is_dropdown
        && (has_icon_attribute(metadata)
            || (metadata.component == "IconButton" && metrics.icon_size > 0.0));
    let trailing_extra_width = if is_dropdown {
        dropdown_adornment_width(metadata)
    } else {
        0.0
    };
    Some(metrics.apply_to_content(content, has_icon, trailing_extra_width))
}

// Keep the measured text lane wide enough for the same native adornments
// reserved by the Dropdown painter, including its token-derived defaults.
/// 返回 Dropdown 尾部四项装饰度量之和；只在调用期间读取 `metadata`。
/// 同名样式覆盖一旦存在便遮蔽属性值，即使覆盖值无效也直接回退设计令牌，不再尝试属性值。
/// caret 尺寸要求为正，其他三项允许零；这些比较不排除正无穷。
fn dropdown_adornment_width(metadata: &UiTemplateNodeMetadata) -> f32 {
    let tokens = EditorDesignTokens::workbench_dark();
    let metric = |key: &str, fallback: f32, positive: bool| {
        metadata
            .style_overrides
            .get(key)
            .or_else(|| metadata.attributes.get(key))
            .and_then(value_as_f32)
            .filter(|value| {
                if positive {
                    *value > 0.0
                } else {
                    *value >= 0.0
                }
            })
            .unwrap_or(fallback)
    };
    metric("horizontal_inset", tokens.density.gap_medium, false)
        + metric("caret_right_inset", tokens.density.gap_large, false)
        + metric("caret_size", tokens.typography.overlay_size, true)
        + metric("caret_gap", tokens.density.gap_small, false)
}

fn supports_material_layout(component: &str) -> bool {
    matches!(
        component,
        "Button"
            | "IconButton"
            | "ToggleButton"
            | "Checkbox"
            | "InputField"
            | "TextField"
            | "ListRow"
            | "ComboBox"
            | "Dropdown"
            | "RangeField"
            | "NumberField"
            | "Progress"
            | "ProgressBar"
            | "LinearProgress"
            | "CircularProgress"
            | "Spinner"
            | "Skeleton"
            | "Backdrop"
            | "Paper"
            | "Modal"
            | "Dialog"
            | "AlertDialog"
            | "Popover"
            | "Popper"
            | "Tooltip"
            | "Snackbar"
            | "Menu"
            | "Drawer"
            | "Switch"
            | "ContextActionMenu"
            | "MenuItem"
            | "Tab"
            | "TableRow"
            | "VirtualList"
            | "ColorField"
            | "Vector2Field"
            | "Vector3Field"
            | "Vector4Field"
            | "Label"
    )
}

fn has_icon_attribute(metadata: &UiTemplateNodeMetadata) -> bool {
    ["icon", "image", "media", "source"].iter().any(|key| {
        metadata
            .attributes
            .get(*key)
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.is_empty())
    })
}

/// 将 TOML Float/Integer 转为 `f32`；不拒绝非有限值，也不设范围上限。
fn value_as_f32(value: &Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}

#[cfg(test)]
#[path = "material/tests/single_pass_tests.rs"]
mod single_pass_tests;
