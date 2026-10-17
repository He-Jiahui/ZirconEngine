use super::{ComponentPropertyDescriptor, ComponentTypeDescriptor};

impl ComponentPropertyDescriptor {
    /// 构造拥有字段名和类型文本的声明，此时尚未注册字段或检查值。
    ///
    /// # 参数与返回值
    /// `name` 为字段键，`value_type` 为类型文本，`editable` 为编辑权限声明。
    /// 文本转为拥有型字符串后返回元数据，合法性由后续注册及反射写入入口判断。
    pub fn new(name: impl Into<String>, value_type: impl Into<String>, editable: bool) -> Self {
        Self {
            name: name.into(),
            value_type: value_type.into(),
            editable,
        }
    }
}

impl ComponentTypeDescriptor {
    /// 构造空属性列表的组件声明，接管三个字符串参数。
    ///
    /// # 参数与返回值
    /// `type_id` 为完整类型路径，`plugin_id` 为所属插件，`display_name` 为显示文本。
    /// 返回尚未发布的描述符；插件前缀、路径重名及字段准入由 World 注册链检查。
    pub fn new(
        type_id: impl Into<String>,
        plugin_id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            type_id: type_id.into(),
            plugin_id: plugin_id.into(),
            display_name: display_name.into(),
            properties: Vec::new(),
        }
    }

    /// 消费当前描述符并追加一个字段声明，返回可继续构造的描述符。
    ///
    /// # 参数与约束
    /// 参数含义同字段声明构造器；本方法按追加顺序保留重复名，也不解析类型文本。
    /// 发布前检查见 `World::register_component_type` 及其反射注册链。
    pub fn with_property(
        mut self,
        name: impl Into<String>,
        value_type: impl Into<String>,
        editable: bool,
    ) -> Self {
        self.properties
            .push(ComponentPropertyDescriptor::new(name, value_type, editable));
        self
    }
}
