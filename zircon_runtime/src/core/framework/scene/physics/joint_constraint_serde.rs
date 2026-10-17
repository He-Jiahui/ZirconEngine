use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserializer, Serializer};

use crate::core::math::Real;

use super::PhysicsJointDrive;

const AXIS_LIMIT_KEYS: [&str; 3] = ["x", "y", "z"];

/// 供 serde 判断是否省略整个限位字段，仅当三个槽位都缺失时为真。
pub(super) fn axis_limits_are_empty(limits: &[Option<[Real; 2]>; 3]) -> bool {
    limits.iter().all(Option::is_none)
}

/// 供 serde 判断是否省略整个驱动字段，逐轴比较全部驱动成员的默认值。
pub(super) fn joint_drives_are_default(drives: &[PhysicsJointDrive; 3]) -> bool {
    drives
        .iter()
        .all(|drive| *drive == PhysicsJointDrive::default())
}

// 持久化时用命名轴稀疏映射；回读同时接纳旧的三槽数组表示。
/// 将借用的三个轴槽位同步写成命名轴稀疏映射。
///
/// # 参数与返回值
/// `limits` 按 x/y/z 索引，缺失槽位不写出，返回序列化器结果或其错误。
/// 本方法不验证上下限顺序或浮点有限性，数值准入另由消费方处理。
pub(super) fn serialize_axis_limits<S>(
    limits: &[Option<[Real; 2]>; 3],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let entry_count = limits.iter().filter(|limit| limit.is_some()).count();
    let mut map = serializer.serialize_map(Some(entry_count))?;
    for (index, limit) in limits.iter().enumerate() {
        if let Some(limit) = limit {
            map.serialize_entry(AXIS_LIMIT_KEYS[index], limit)?;
        }
    }
    map.end()
}

/// 把兼容的轴文档恢复为拥有型三槽数组，不保留反序列化输入的借用。
///
/// # 约束与错误
/// 格式须支持 `deserialize_any`，序列/映射的结构检查由 Visitor 执行，数值准入不在这里进行。
/// 见 `visit_seq`、`visit_map` 及稀疏轴文档测试。
pub(super) fn deserialize_axis_limits<'de, D>(
    deserializer: D,
) -> Result<[Option<[Real; 2]>; 3], D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(AxisLimitsVisitor)
}

struct AxisLimitsVisitor;

impl<'de> Visitor<'de> for AxisLimitsVisitor {
    type Value = [Option<[Real; 2]>; 3];

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("three optional axis limits or an x/y/z limit map")
    }

    /// 按槽位读取零至三个可空限位，提前结束时其余槽位保持缺失。
    /// 第四个元素导致长度错误；元素解码错误直接传播，不返回部分数组。
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut limits = [None, None, None];
        for slot in &mut limits {
            if let Some(limit) = sequence.next_element::<Option<[Real; 2]>>()? {
                *slot = limit;
            } else {
                return Ok(limits);
            }
        }
        if sequence.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(4, &"at most three axis limits"));
        }
        Ok(limits)
    }

    /// 接纳命名轴及数字别名，并按规范轴槽位拒绝重名。
    /// 未知键、重复轴或值解码错误直接失败；映射值必须是两元素范围，缺失轴以不写键表示。
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut limits = [None, None, None];
        let mut seen = [false; 3];
        while let Some(key) = map.next_key::<String>()? {
            let index = axis_limit_index(&key)
                .ok_or_else(|| de::Error::unknown_field(&key, &AXIS_LIMIT_KEYS))?;
            if seen[index] {
                return Err(de::Error::duplicate_field(AXIS_LIMIT_KEYS[index]));
            }
            seen[index] = true;
            limits[index] = Some(map.next_value()?);
        }
        Ok(limits)
    }
}

/// 将小写命名轴或数字别名映射到固定槽位，不裁剪文本，未知键返回 `None`。
fn axis_limit_index(key: &str) -> Option<usize> {
    match key {
        "x" | "0" => Some(0),
        "y" | "1" => Some(1),
        "z" | "2" => Some(2),
        _ => None,
    }
}
