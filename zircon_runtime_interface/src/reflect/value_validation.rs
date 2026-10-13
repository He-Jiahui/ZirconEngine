//! 以显式工作栈遍历反射值与嵌入 JSON 树，并在调用方配额内验证。
//!
//! 检查发生在值已构造之后；JSON 文本解析、字段 schema、Rust 类型匹配与权限由各自边界处理。

use serde_json::Value;

use super::{
    ReflectValueBudget, ReflectValueBudgetDimension, ReflectValueFloatKind,
    ReflectValueValidationError, ReflectedValue,
};

/// 当前同步遍历中暂存的借用节点及其一基深度。
enum PendingValue<'a> {
    /// 一个反射包装值和从根起算的一基深度。
    Reflected {
        /// 借用的反射节点；工作项不会越过当前验证调用。
        value: &'a ReflectedValue,
        /// 根节点为 1，每经过一个父容器成员深度加 1。
        depth: usize,
    },
    /// ReflectedValue::Json 内的 JSON 节点；JSON 根是包装值的子节点。
    Json {
        /// 从 Json 包装值借用、只在当前遍历中使用的节点。
        value: &'a Value,
        /// 包含 Json 包装值和 JSON 根的一基深度。
        depth: usize,
    },
}

// 显式 Vec 工作栈避免递归；借用覆盖当前调用，不复制或改写输入值。

impl ReflectedValue {
    /// 检查含嵌入 JSON 子树的完整值；字符串 UTF-8 字节（含映射键）也计入预算，同时拒绝非有限浮点。
    ///
    /// # 使用时机与生命周期
    /// 在调用者取得 ReflectedValue 后执行；Runtime 场景和 VM 入口传入自己的配额。本方法只借用 self。
    ///
    /// # 参数
    /// budget 是本次遍历共用的深度、节点、字符串字节及单容器条目上限。
    ///
    /// # 返回值
    /// 全部反射与嵌入 JSON 节点通过时返回 Ok；首次越界、计数溢出或非有限反射浮点返回对应错误。
    ///
    /// # 约束
    /// 根深度为 1；Json 包装节点和 JSON 根分别计节点，JSON 根深度加 1。键字符串在子值访问前累计。
    /// 本方法不限制此前的 JSON 字节解析/分配，也不检查四元数归一化、资源 URI、schema、字段类型或权限。
    ///
    /// # 参考
    /// Runtime 调用点包括 scene/reflect/value_admission.rs、scene/reflect/json_document/read.rs、
    /// scene/reflect/json_document/write.rs 和 script/vm/plugin 状态载入及迁移；契约见本 crate 的预算测试。

    pub fn validate_with_budget(
        &self,
        budget: ReflectValueBudget,
    ) -> Result<(), ReflectValueValidationError> {
        // 每次调用从零累计；失败路径丢弃局部状态，不改变原值。
        let mut state = ValidationState::default();
        let mut pending = vec![PendingValue::Reflected {
            value: self,
            depth: 1,
        }];

        // 反向入栈的子项由 LIFO 按容器顺序弹出；这是显式工作栈，不是递归调用栈。
        // 节点额度不承诺工作栈宽度或内存上限；? 在首个错误处结束检查。
        while let Some(value) = pending.pop() {
            match value {
                PendingValue::Reflected { value, depth } => {
                    state.visit(depth, budget)?;
                    validate_reflected(value, depth, budget, &mut state, &mut pending)?;
                }
                PendingValue::Json { value, depth } => {
                    state.visit(depth, budget)?;
                    validate_json(value, depth, budget, &mut state, &mut pending)?;
                }
            }
        }
        Ok(())
    }
}

/// 检查一个反射包装节点并把子节点加入工作栈。
///
/// # 参数
/// value 和 pending 借用当前输入；depth 是一基深度；budget 与 state 分别是共享上限和累计量。
///
/// # 返回值
/// 当前载荷检查通过并排入子项时返回 Ok，失败时返回当前节点错误。
///
/// # 约束
/// 不递归调用自身；Map 键先计字节再排值，Json 根是包装节点的子节点。

fn validate_reflected<'a>(
    value: &'a ReflectedValue,
    depth: usize,
    budget: ReflectValueBudget,
    state: &mut ValidationState,
    pending: &mut Vec<PendingValue<'a>>,
) -> Result<(), ReflectValueValidationError> {
    match value {
        // 标量、向量和四元数只检查分量有限性，不检查归一化或领域约束。
        ReflectedValue::Scalar(value) => {
            validate_finite(ReflectValueFloatKind::Scalar, std::slice::from_ref(value))
        }
        ReflectedValue::Vec2(values) => validate_finite(ReflectValueFloatKind::Vec2, values),
        ReflectedValue::Vec3(values) => validate_finite(ReflectValueFloatKind::Vec3, values),
        ReflectedValue::Vec4(values) => validate_finite(ReflectValueFloatKind::Vec4, values),
        ReflectedValue::Quaternion(values) => {
            validate_finite(ReflectValueFloatKind::Quaternion, values)
        }
        // String、Enum、Resource 文本计 UTF-8 字节；不解析资源 URI。
        ReflectedValue::String(value)
        | ReflectedValue::Enum(value)
        | ReflectedValue::Resource(value) => state.add_string(value, budget),
        // List 先比较本容器条目数，再为非空子项计算深度并入栈。
        ReflectedValue::List(values) => {
            validate_container(values.len(), budget)?;
            if !values.is_empty() {
                let depth = child_depth(depth, budget)?;
                pending.extend(
                    values
                        .iter()
                        .rev()
                        .map(|value| PendingValue::Reflected { value, depth }),
                );
            }
            Ok(())
        }
        // Map 键值对逐容器受限；键先计字符串额度，再访问其值。
        ReflectedValue::Map(values) => {
            validate_container(values.len(), budget)?;
            for key in values.keys() {
                state.add_string(key, budget)?;
            }
            if !values.is_empty() {
                let depth = child_depth(depth, budget)?;
                pending.extend(
                    values
                        .values()
                        .rev()
                        .map(|value| PendingValue::Reflected { value, depth }),
                );
            }
            Ok(())
        }
        // JSON 根作为 Json 包装值的子节点，占下一层深度和一个节点额度。
        ReflectedValue::Json(value) => {
            pending.push(PendingValue::Json {
                value,
                depth: child_depth(depth, budget)?,
            });
            Ok(())
        }
        // 这些标量不增加字符串字节或容器条目；字段类型契约在别处处理。
        ReflectedValue::Null
        | ReflectedValue::Bool(_)
        | ReflectedValue::Integer(_)
        | ReflectedValue::Unsigned(_)
        | ReflectedValue::Entity(_) => Ok(()),
    }
}

/// 检查一个已构造的 serde_json 节点并把容器子项压入工作栈。
///
/// # 参数
/// value 是 JSON 节点；depth、budget、state 和 pending 分别为深度、共享上限、累计量和待处理借用项。
///
/// # 返回值
/// 当前 JSON 节点自身检查通过且后代已排队时返回 Ok；预算错误立即返回。
///
/// # 约束
/// 对象键与字符串值计入 UTF-8 字节，数组和对象逐个检查直接条目数；本函数不解析 JSON 文本。

fn validate_json<'a>(
    value: &'a Value,
    depth: usize,
    budget: ReflectValueBudget,
    state: &mut ValidationState,
    pending: &mut Vec<PendingValue<'a>>,
) -> Result<(), ReflectValueValidationError> {
    match value {
        // JSON 字符串与反射字符串共用本次调用的字节累计。
        Value::String(value) => state.add_string(value, budget),
        // 数组长度单独检查；非空时才计算子深度并入栈。
        Value::Array(values) => {
            validate_container(values.len(), budget)?;
            if !values.is_empty() {
                let depth = child_depth(depth, budget)?;
                pending.extend(
                    values
                        .iter()
                        .rev()
                        .map(|value| PendingValue::Json { value, depth }),
                );
            }
            Ok(())
        }
        // 对象键先进入字符串累计，值随后作为子节点入栈。
        Value::Object(values) => {
            validate_container(values.len(), budget)?;
            for key in values.keys() {
                state.add_string(key, budget)?;
            }
            if !values.is_empty() {
                let depth = child_depth(depth, budget)?;
                pending.extend(
                    values
                        .values()
                        .rev()
                        .map(|value| PendingValue::Json { value, depth }),
                );
            }
            Ok(())
        }
        // JSON 叶节点只占已计入的节点，不经过反射浮点有限性检查。
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(()),
    }
}

/// 一次 validate_with_budget 调用的局部累计量；调用之间不共享。
#[derive(Default)]
struct ValidationState {
    /// 已通过上限检查的反射及 JSON 节点数。
    nodes: usize,
    /// 已通过上限检查的字符串及键的 UTF-8 字节数。
    string_bytes: usize,
}

// 先检查下一累计值，只有成功时才更新状态。

impl ValidationState {
    /// 先检查深度，再尝试累计当前节点；错误时不提交本次计数。
    fn visit(
        &mut self,
        depth: usize,
        budget: ReflectValueBudget,
    ) -> Result<(), ReflectValueValidationError> {
        // 同一节点同时越过深度和节点限制时先返回深度错误。
        if depth > budget.max_depth() {
            return Err(ReflectValueValidationError::DepthExceeded {
                actual: depth,
                maximum: budget.max_depth(),
            });
        }
        let actual = self.nodes.checked_add(1).ok_or(
            ReflectValueValidationError::BudgetArithmeticOverflow {
                dimension: ReflectValueBudgetDimension::Nodes,
            },
        )?;
        if actual > budget.max_nodes() {
            return Err(ReflectValueValidationError::NodeBudgetExceeded {
                actual,
                maximum: budget.max_nodes(),
            });
        }
        // 只有新节点计数通过上限才提交。
        self.nodes = actual;
        Ok(())
    }

    /// 累计反射文本、容器键或 JSON 字符串的 UTF-8 字节，并仅在成功时提交。
    fn add_string(
        &mut self,
        value: &str,
        budget: ReflectValueBudget,
    ) -> Result<(), ReflectValueValidationError> {
        // str::len 是 UTF-8 字节长度；先 checked_add，再比较上限，最后提交。
        let actual = self.string_bytes.checked_add(value.len()).ok_or(
            ReflectValueValidationError::BudgetArithmeticOverflow {
                dimension: ReflectValueBudgetDimension::StringBytes,
            },
        )?;
        if actual > budget.max_string_bytes() {
            return Err(ReflectValueValidationError::StringBudgetExceeded {
                actual,
                maximum: budget.max_string_bytes(),
            });
        }
        self.string_bytes = actual;
        Ok(())
    }
}

/// 将单个容器的直接长度与 max_container_entries 比较；不同容器不累计在一起。
fn validate_container(
    actual: usize,
    budget: ReflectValueBudget,
) -> Result<(), ReflectValueValidationError> {
    if actual > budget.max_container_entries() {
        Err(ReflectValueValidationError::ContainerEntriesExceeded {
            actual,
            maximum: budget.max_container_entries(),
        })
    } else {
        Ok(())
    }
}

/// 对子节点做 checked_add 和深度比较，在入栈前返回下一层深度或错误。
fn child_depth(
    depth: usize,
    budget: ReflectValueBudget,
) -> Result<usize, ReflectValueValidationError> {
    let actual =
        depth
            .checked_add(1)
            .ok_or(ReflectValueValidationError::BudgetArithmeticOverflow {
                dimension: ReflectValueBudgetDimension::Depth,
            })?;
    if actual > budget.max_depth() {
        Err(ReflectValueValidationError::DepthExceeded {
            actual,
            maximum: budget.max_depth(),
        })
    } else {
        Ok(actual)
    }
}

/// 查找反射浮点数组的首个非有限分量，并以零基索引返回错误。
fn validate_finite(
    kind: ReflectValueFloatKind,
    values: &[f32],
) -> Result<(), ReflectValueValidationError> {
    if let Some((component, _)) = values
        .iter()
        .enumerate()
        .find(|(_, value)| !value.is_finite())
    {
        Err(ReflectValueValidationError::NonFiniteFloat { kind, component })
    } else {
        Ok(())
    }
}
