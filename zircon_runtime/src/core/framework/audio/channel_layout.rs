//! 定义资产、混音数据和声音服务共享的声道数量与扬声器位置契约。
//!
//! 布局是可拥有、可序列化的元数据，不管理设备、样本缓冲区或播放生命周期。
//! 标准名称约束扬声器顺序，离散布局仅表达数量，自定义布局须显式携带互异的位置。
//! 公开字段和反序列化都能绕过构造器；外部输入交给服务前须校验完整契约，
//! 随后再检查具体后端能力，通用校验通过不等于设备能够输出该布局。
use serde::{Deserialize, Serialize};

/// 保存内建命名布局的枚举词汇，供名称查询和命名布局往返测试读取。
///
/// # 约束与生命周期
///
/// 元素与名称解析、静态契约表的八种预设保持一致；此列表没有后端能力筛选作用。
/// 借用的数据随程序存在，查询不构造布局，也不分配名称或扬声器数组。
///
/// # 参考
///
/// 参见 [`AudioChannelLayout::named_layout_names`] 与 [`AudioChannelLayout::from_name`]。
const NAMED_CHANNEL_LAYOUT_NAMES: &[&str] = &[
    "mono",
    "stereo",
    "quad",
    "surround_5_0",
    "surround_5_1",
    "surround_5_1_side",
    "surround_7_0",
    "surround_7_1",
];

/// 表示交错音频帧中一个通道的扬声器位置语义。
///
/// # 使用时机与约束
///
/// WAV 资产从受支持的通道掩码映射这些位置；布局、声音选项与设备描述符传递同一语义。
/// 枚举值本身不表示帧中索引，索引由 [`AudioChannelLayout::speakers`] 的顺序决定。
/// 序列化使用蛇形名称；内部去重位与 WAV 掩码属于不同表示，不能互相替代。
///
/// # 生命周期与参考
///
/// 值可以复制，不拥有设备或声源资源；顺序和互异性由布局校验负责。
/// 参见 [`AudioChannelLayout`] 与 [`AudioChannelLayout::has_unique_speakers`]。
/// WAV 掩码、声音资产与设备描述符共用的扬声器语义；顺序必须由布局契约保留。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioSpeakerChannel {
    /// 前左位置，标准立体声布局的第一个通道。
    ///
    /// 位置相同不代表当前帧索引相同；以所属布局的扬声器顺序为准。
    FrontLeft,
    /// 前右位置，标准立体声布局的第二个通道。
    ///
    /// 与前左通道区分左右语义；自定义布局仍须显式保留其索引关系。
    FrontRight,
    /// 前中位置，内建单声道布局采用这一位置。
    ///
    /// 它是扬声器元数据，不承诺后端已完成多通道合成或中置路由。
    FrontCenter,
    /// 低频效果位置，内建 5.1 与 7.1 布局包含这一通道。
    ///
    /// 标记不执行滤波或分频；样本顺序仍由所属布局确定。
    LowFrequency,
    /// 后左位置，后环绕预设采用这一位置。
    ///
    /// 与侧左是不同语义；六声道的后环绕和侧环绕预设不可仅凭数量替换。
    BackLeft,
    /// 后右位置，后环绕预设采用这一位置。
    ///
    /// 与侧右是不同语义；校验内建布局时会核对位置和排列次序。
    BackRight,
    /// 侧左位置，侧环绕及七通道、八通道预设采用这一位置。
    ///
    /// 不以枚举内部去重位编码 WAV 掩码；资产解析承担外部掩码映射。
    SideLeft,
    /// 侧右位置，侧环绕及七通道、八通道预设采用这一位置。
    ///
    /// 位置值可复制，没有独立的设备生命周期或输出路由状态。
    SideRight,
}

/// 查找内建名称对应的数量和有序扬声器契约，供校验路径借用静态元数据。
///
/// # 参数
///
/// - `name`：待匹配的原始名称；只接受八个精确名称，不修剪空白或转换大小写。
///
/// # 返回值
///
/// 命中时返回数量和静态位置切片；未知、自定义或离散名称返回 `None`。
/// 返回的切片不拥有临时布局，比较时无需分配名称字符串或位置数组。
///
/// # 使用时机与参考
///
/// 完整校验先用此查询识别保留名称，再验证数量与顺序，防止保留名称冒充自定义布局。
/// 参见 [`AudioChannelLayout::is_canonical_named_layout`] 与 [`AudioChannelLayout::is_valid_contract_layout`]。
fn named_layout_contract(name: &str) -> Option<(u16, &'static [AudioSpeakerChannel])> {
    use AudioSpeakerChannel::{
        BackLeft, BackRight, FrontCenter, FrontLeft, FrontRight, LowFrequency, SideLeft, SideRight,
    };

    match name {
        "mono" => Some((1, &[FrontCenter])),
        "stereo" => Some((2, &[FrontLeft, FrontRight])),
        "quad" => Some((4, &[FrontLeft, FrontRight, BackLeft, BackRight])),
        "surround_5_0" => Some((
            5,
            &[FrontLeft, FrontRight, FrontCenter, BackLeft, BackRight],
        )),
        "surround_5_1" => Some((
            6,
            &[
                FrontLeft,
                FrontRight,
                FrontCenter,
                LowFrequency,
                BackLeft,
                BackRight,
            ],
        )),
        "surround_5_1_side" => Some((
            6,
            &[
                FrontLeft,
                FrontRight,
                FrontCenter,
                LowFrequency,
                SideLeft,
                SideRight,
            ],
        )),
        "surround_7_0" => Some((
            7,
            &[
                FrontLeft,
                FrontRight,
                FrontCenter,
                BackLeft,
                BackRight,
                SideLeft,
                SideRight,
            ],
        )),
        "surround_7_1" => Some((
            8,
            &[
                FrontLeft,
                FrontRight,
                FrontCenter,
                LowFrequency,
                BackLeft,
                BackRight,
                SideLeft,
                SideRight,
            ],
        )),
        _ => None,
    }
}

/// 比较离散名称的规范十进制后缀与声明数量，供重复校验时直接借用名称字节。
///
/// # 参数
///
/// - `name`：完整名称，须有 `discrete_` 前缀和非空 ASCII 十进制后缀；多位数不接受前导零。
/// - `channel_count`：比较目标，允许零；正声道数约束由外层布局校验承担。
///
/// # 返回值与约束
///
/// 只有后缀无符号、无额外字符、可表示为 `u16` 且等于目标时返回真；溢出返回假。
/// 本函数对 `discrete_0` 和零返回真，但这不表示零声道布局有效。
/// 借用输入只覆盖本次调用，不构造临时名称，也不改变布局。
///
/// # 参考
///
/// 参见 [`AudioChannelLayout::is_canonical_discrete_layout`] 与本模块的离散名称契约测试。
fn canonical_discrete_name_matches(name: &str, channel_count: u16) -> bool {
    let Some(suffix) = name.strip_prefix("discrete_") else {
        return false;
    };
    if suffix.is_empty() || (suffix.len() > 1 && suffix.as_bytes()[0] == b'0') {
        return false;
    }

    let mut parsed = 0_u16;
    for byte in suffix.bytes() {
        if !byte.is_ascii_digit() {
            return false;
        }
        let digit = u16::from(byte - b'0');
        let Some(next) = parsed
            .checked_mul(10)
            .and_then(|value| value.checked_add(digit))
        else {
            return false;
        };
        parsed = next;
    }
    parsed == channel_count
}

/// 把一个扬声器位置映射到内部去重位，使位置列表能用一次遍历检查重复。
///
/// # 参数与返回值
///
/// - `speaker`：当前已定义的八种位置之一；返回 `u8` 中互不重叠的单个位。
///
/// # 约束与参考
///
/// 该映射只服务本地集合成员判断，没有持久化或 WAV 掩码含义；侧位置的外部掩码另行解析。
/// 新增位置时须同时审视位宽与去重契约。参见 [`AudioChannelLayout::has_unique_speakers`]。
fn speaker_bit(speaker: AudioSpeakerChannel) -> u8 {
    match speaker {
        AudioSpeakerChannel::FrontLeft => 1 << 0,
        AudioSpeakerChannel::FrontRight => 1 << 1,
        AudioSpeakerChannel::FrontCenter => 1 << 2,
        AudioSpeakerChannel::LowFrequency => 1 << 3,
        AudioSpeakerChannel::BackLeft => 1 << 4,
        AudioSpeakerChannel::BackRight => 1 << 5,
        AudioSpeakerChannel::SideLeft => 1 << 6,
        AudioSpeakerChannel::SideRight => 1 << 7,
    }
}

/// 拥有音频格式的名称、声道数量与有序扬声器位置，作为资产和声音服务之间的格式契约。
///
/// # 使用时机
///
/// 导入器保存格式元数据，混音块及设备、外部声源描述符用它关联交错帧的通道意义。
/// 内建构造器提供标准顺序；无位置语义的输入使用离散布局，自定义布局须显式给出位置。
///
/// # 必要约束
///
/// 有效契约要求正数量、规范名称，并满足命名、离散或自定义布局各自的顺序／长度／互异条件。
/// 公开字段与反序列化不自动检查这些条件，接收边界应调用 [`Self::is_valid_contract_layout`]。
/// 设备能力、缓冲区长度和采样率仍需各自检查，此类型的校验不执行重排、转换或打开设备。
///
/// # 生命周期与参考
///
/// 名称和位置数组随值拥有，克隆得到独立元数据；修改布局不会重配已存在的声源或设备。
/// 参见 [`Self::from_name`]、[`Self::discrete`] 与 [`Self::is_valid_contract_layout`]。
/// 声音资产、混音块与输出设备之间传递的通道布局契约。
///
/// 外部构造或反序列化后应校验名称、数量和扬声器顺序；离散布局只表达数量，不携带位置语义。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioChannelLayout {
    /// 格式标识；内建名称及 `discrete_` 前缀保留各自的规范契约。
    ///
    /// 完整校验拒绝空名称和两端空白；自定义名称可用于非空且数量匹配、位置互异的列表。
    /// 字符串由布局拥有，改名不会重排样本，变更后须重新检查完整契约。
    pub name: String,
    /// 每个交错音频帧的通道数量，有效交换契约要求大于零。
    ///
    /// 它与设备或缓冲区的声明数量比较，但仅比较此字段不证明布局的名称和位置正确。
    /// 公开赋值不会同步位置数组；参见 [`AudioChannelLayout::matches_channel_count`] 与 [`AudioChannelLayout::is_valid_contract_layout`]。
    pub channel_count: u16,
    /// 按交错帧通道索引排列的扬声器位置，由布局拥有。
    ///
    /// 空列表只在完整校验接受的规范离散布局中表示位置未知；非空列表须数量匹配且位置互异。
    /// 内建名称还要求固定排列。列表排序或替换不会同步重排已有样本，调用方须维护二者关系。
    pub speakers: Vec<AudioSpeakerChannel>,
}

impl AudioChannelLayout {
    /// 构造 `mono` 的内建布局元数据，供已知单声道格式或单声道配置使用。
    ///
    /// # 返回值
    ///
    /// 返回名称为 `mono`、数量为 1 的布局；交错帧位置顺序为前中。
    ///
    /// # 约束与生命周期
    ///
    /// 结果拥有名称和位置数组，不借用设备或样本；它不验证输入确有这些位置，也不改变样本排列。
    /// 调用方在具体后端边界仍须检查支持能力；修改公开字段后须重新校验契约。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::from_name`]、[`Self::for_channel_count`] 与 [`Self::is_canonical_named_layout`]。
    pub fn mono() -> Self {
        Self {
            name: "mono".to_string(),
            channel_count: 1,
            speakers: vec![AudioSpeakerChannel::FrontCenter],
        }
    }

    /// 构造 `stereo` 的内建布局元数据，供已知立体声格式及声音服务的立体声配置使用。
    ///
    /// # 返回值
    ///
    /// 返回名称为 `stereo`、数量为 2 的布局；交错帧位置顺序为前左、前右。
    ///
    /// # 约束与生命周期
    ///
    /// 结果拥有名称和位置数组，不借用设备或样本；它不验证输入确有这些位置，也不改变样本排列。
    /// 调用方在具体后端边界仍须检查支持能力；修改公开字段后须重新校验契约。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::from_name`]、[`Self::for_channel_count`] 与 [`Self::is_canonical_named_layout`]。
    pub fn stereo() -> Self {
        Self {
            name: "stereo".to_string(),
            channel_count: 2,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
            ],
        }
    }

    /// 构造 `quad` 的内建布局元数据，供明确带后左右位置的四通道格式使用。
    ///
    /// # 返回值
    ///
    /// 返回名称为 `quad`、数量为 4 的布局；交错帧位置顺序为前左、前右、后左、后右。
    ///
    /// # 约束与生命周期
    ///
    /// 结果拥有名称和位置数组，不借用设备或样本；它不验证输入确有这些位置，也不改变样本排列。
    /// 调用方在具体后端边界仍须检查支持能力；修改公开字段后须重新校验契约。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::from_name`]、[`Self::for_channel_count`] 与 [`Self::is_canonical_named_layout`]。
    pub fn quad() -> Self {
        Self {
            name: "quad".to_string(),
            channel_count: 4,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::BackLeft,
                AudioSpeakerChannel::BackRight,
            ],
        }
    }

    /// 构造 `surround_5_1` 的内建布局元数据，供后环绕 5.1 格式及六声道数量回退路径使用。
    ///
    /// # 返回值
    ///
    /// 返回名称为 `surround_5_1`、数量为 6 的布局；交错帧位置顺序为前左、前右、前中、低频、后左、后右。
    ///
    /// # 约束与生命周期
    ///
    /// 结果拥有名称和位置数组，不借用设备或样本；它不验证输入确有这些位置，也不改变样本排列。
    /// 调用方在具体后端边界仍须检查支持能力；修改公开字段后须重新校验契约。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::from_name`]、[`Self::for_channel_count`] 与 [`Self::is_canonical_named_layout`]。
    pub fn surround_5_1() -> Self {
        Self {
            name: "surround_5_1".to_string(),
            channel_count: 6,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontCenter,
                AudioSpeakerChannel::LowFrequency,
                AudioSpeakerChannel::BackLeft,
                AudioSpeakerChannel::BackRight,
            ],
        }
    }

    /// 构造 `surround_5_0` 的内建布局元数据，供明确不带低频位置的五通道格式使用。
    ///
    /// # 返回值
    ///
    /// 返回名称为 `surround_5_0`、数量为 5 的布局；交错帧位置顺序为前左、前右、前中、后左、后右。
    ///
    /// # 约束与生命周期
    ///
    /// 结果拥有名称和位置数组，不借用设备或样本；它不验证输入确有这些位置，也不改变样本排列。
    /// 调用方在具体后端边界仍须检查支持能力；修改公开字段后须重新校验契约。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::from_name`]、[`Self::for_channel_count`] 与 [`Self::is_canonical_named_layout`]。
    pub fn surround_5_0() -> Self {
        Self {
            name: "surround_5_0".to_string(),
            channel_count: 5,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontCenter,
                AudioSpeakerChannel::BackLeft,
                AudioSpeakerChannel::BackRight,
            ],
        }
    }

    /// 构造 `surround_5_1_side` 的内建布局元数据，供已有侧环绕位置证据的 5.1 格式使用。
    ///
    /// # 返回值
    ///
    /// 返回名称为 `surround_5_1_side`、数量为 6 的布局；交错帧位置顺序为前左、前右、前中、低频、侧左、侧右。
    ///
    /// # 约束与生命周期
    ///
    /// 结果拥有名称和位置数组，不借用设备或样本；它不验证输入确有这些位置，也不改变样本排列。
    /// 调用方在具体后端边界仍须检查支持能力；修改公开字段后须重新校验契约。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::from_name`]、[`Self::for_channel_count`] 与 [`Self::is_canonical_named_layout`]。
    pub fn surround_5_1_side() -> Self {
        Self {
            name: "surround_5_1_side".to_string(),
            channel_count: 6,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontCenter,
                AudioSpeakerChannel::LowFrequency,
                AudioSpeakerChannel::SideLeft,
                AudioSpeakerChannel::SideRight,
            ],
        }
    }

    /// 构造 `surround_7_0` 的内建布局元数据，供明确不带低频位置的七通道格式使用。
    ///
    /// # 返回值
    ///
    /// 返回名称为 `surround_7_0`、数量为 7 的布局；交错帧位置顺序为前左、前右、前中、后左、后右、侧左、侧右。
    ///
    /// # 约束与生命周期
    ///
    /// 结果拥有名称和位置数组，不借用设备或样本；它不验证输入确有这些位置，也不改变样本排列。
    /// 调用方在具体后端边界仍须检查支持能力；修改公开字段后须重新校验契约。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::from_name`]、[`Self::for_channel_count`] 与 [`Self::is_canonical_named_layout`]。
    pub fn surround_7_0() -> Self {
        Self {
            name: "surround_7_0".to_string(),
            channel_count: 7,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontCenter,
                AudioSpeakerChannel::BackLeft,
                AudioSpeakerChannel::BackRight,
                AudioSpeakerChannel::SideLeft,
                AudioSpeakerChannel::SideRight,
            ],
        }
    }

    /// 构造 `surround_7_1` 的内建布局元数据，供明确包含后环绕和侧环绕的八通道格式使用。
    ///
    /// # 返回值
    ///
    /// 返回名称为 `surround_7_1`、数量为 8 的布局；交错帧位置顺序为前左、前右、前中、低频、后左、后右、侧左、侧右。
    ///
    /// # 约束与生命周期
    ///
    /// 结果拥有名称和位置数组，不借用设备或样本；它不验证输入确有这些位置，也不改变样本排列。
    /// 调用方在具体后端边界仍须检查支持能力；修改公开字段后须重新校验契约。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::from_name`]、[`Self::for_channel_count`] 与 [`Self::is_canonical_named_layout`]。
    pub fn surround_7_1() -> Self {
        Self {
            name: "surround_7_1".to_string(),
            channel_count: 8,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontCenter,
                AudioSpeakerChannel::LowFrequency,
                AudioSpeakerChannel::BackLeft,
                AudioSpeakerChannel::BackRight,
                AudioSpeakerChannel::SideLeft,
                AudioSpeakerChannel::SideRight,
            ],
        }
    }

    /// 构造只记录数量、不声明扬声器位置的离散布局，供未知位置语义的音频格式使用。
    ///
    /// # 参数
    ///
    /// - `channel_count`：按值保存的通道数量；构造器允许零，有效交换契约要求调用方提供正数量。
    ///
    /// # 返回值与生命周期
    ///
    /// 返回拥有规范十进制名称 `discrete_数量` 的布局，位置列表为空，不推断环绕顺序。
    /// 零数量也能构造，但不能通过 [`Self::is_valid_contract_layout`]；构造不验证样本或后端能力。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::is_canonical_discrete_layout`]；已有可靠位置语义时应保留显式位置元数据。
    pub fn discrete(channel_count: u16) -> Self {
        Self {
            name: format!("discrete_{channel_count}"),
            channel_count,
            speakers: Vec::new(),
        }
    }

    /// 在缺少扬声器位置元数据的数量回退路径中选择引擎约定的预设。
    ///
    /// # 参数
    ///
    /// - `channel_count`：输入格式声明的数量；本方法不检查帧长度，也不证明其实际扬声器位置。
    ///
    /// # 返回值与约束
    ///
    /// 数量 1、2、4、5、6、7、8 对应内建布局，其他数量返回离散布局；六声道采用后环绕 5.1。
    /// 已知侧环绕或其他排列时须保留其显式语义，不能只凭数量回退；零数量的返回值仍是无效契约。
    ///
    /// # 生命周期与参考
    ///
    /// 返回独立元数据，不重排或混合样本。参见 [`Self::surround_5_1_side`] 与 [`Self::discrete`]。
    /// 无扬声器掩码时采用约定的标准布局；六声道默认后环绕，已知侧环绕需显式选择。
    pub fn for_channel_count(channel_count: u16) -> Self {
        match channel_count {
            1 => Self::mono(),
            2 => Self::stereo(),
            4 => Self::quad(),
            5 => Self::surround_5_0(),
            6 => Self::surround_5_1(),
            7 => Self::surround_7_0(),
            8 => Self::surround_7_1(),
            _ => Self::discrete(channel_count),
        }
    }

    /// 把内建布局的精确名称解析为拥有完整位置顺序的元数据，供配置入口选择预设。
    ///
    /// # 参数
    ///
    /// - `name`：原始输入名称；按大小写精确匹配，不修剪空白，允许传入未知名称。
    ///
    /// # 返回值
    ///
    /// 八种内建名称返回对应布局；离散、自定义或未知名称返回 `None`。
    /// `None` 只表示此预设解析器不识别输入，不证明一个显式自定义布局违反完整契约。
    ///
    /// # 生命周期与参考
    ///
    /// 成功结果拥有名称和位置数组，不借用输入；解析不打开设备或保证后端支持。
    /// 参见 [`Self::named_layout_names`]、[`Self::discrete`] 与 [`Self::is_valid_contract_layout`]。
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "mono" => Some(Self::mono()),
            "stereo" => Some(Self::stereo()),
            "quad" => Some(Self::quad()),
            "surround_5_0" => Some(Self::surround_5_0()),
            "surround_5_1" => Some(Self::surround_5_1()),
            "surround_5_1_side" => Some(Self::surround_5_1_side()),
            "surround_7_0" => Some(Self::surround_7_0()),
            "surround_7_1" => Some(Self::surround_7_1()),
            _ => None,
        }
    }

    /// 枚举名称解析器提供的八种内建预设，供名称往返和标准契约测试使用。
    ///
    /// # 返回值与生命周期
    ///
    /// 返回借用的静态名称切片，顺序固定在本模块词汇表中，查询不分配布局。
    /// 它不包含离散或自定义名称，也不筛选当前后端可用格式；具体配置入口可提供更小的选择集。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::from_name`] 与声音布局的内建名称往返测试。
    pub fn named_layout_names() -> &'static [&'static str] {
        NAMED_CHANNEL_LAYOUT_NAMES
    }

    /// 检查另一份格式声明的数量是否与本布局数量相同，供设备及外部声源边界交叉核对。
    ///
    /// # 参数与返回值
    ///
    /// - `channel_count`：另一份声明的数量；两者相等返回真，包括两者都是零的情况。
    /// - `self`：借用布局；此比较不检查名称、位置列表或实际样本长度。
    ///
    /// # 使用时机与参考
    ///
    /// 数量比较应与完整契约及所在边界的缓冲区／后端检查一起使用，不修改任何格式状态。
    /// 参见 [`Self::is_valid_contract_layout`] 与 [`Self::has_matching_speaker_count`]。
    pub fn matches_channel_count(&self, channel_count: u16) -> bool {
        self.channel_count == channel_count
    }

    /// 检查显式位置列表的长度是否与声明数量匹配，作为自定义布局的局部校验。
    ///
    /// # 返回值与约束
    ///
    /// 位置列表为空时返回真，否则只有列表长度等于数量时返回真。
    /// 空列表的真值仅表示没有长度冲突，不证明名称是离散布局，也不排除零数量或重复位置。
    ///
    /// # 生命周期与参考
    ///
    /// 只借用元数据，不填充缺失位置。完整判断使用 [`Self::is_valid_contract_layout`]。
    pub fn has_matching_speaker_count(&self) -> bool {
        self.speakers.is_empty() || self.speakers.len() == usize::from(self.channel_count)
    }

    /// 检查显式位置是否互异，供自定义布局在长度匹配后验证位置集合。
    ///
    /// # 返回值与约束
    ///
    /// 发现重复位置返回假，空列表返回真；此结果不验证名称、数量或内建布局排列顺序。
    /// 检查使用本次调用的本地位集合，一次遍历且不为位置列表分配副本。
    ///
    /// # 生命周期与参考
    ///
    /// 不修改位置或重排样本；参见 [`Self::is_valid_contract_layout`] 与内部的 `speaker_bit`。
    pub fn has_unique_speakers(&self) -> bool {
        let mut seen = 0_u8;
        self.speakers.iter().copied().all(|speaker| {
            let bit = speaker_bit(speaker);
            let unique = seen & bit == 0;
            seen |= bit;
            unique
        })
    }

    /// 验证保留的内建名称是否携带该名称规定的数量和位置顺序。
    ///
    /// # 返回值
    ///
    /// 名称命中且数量、每个位置及其排列全部等于静态契约时返回真；非内建名称返回假。
    /// 同样数量或同样位置集合但顺序不同也返回假，避免交错帧的通道语义被悄悄交换。
    ///
    /// # 使用时机与生命周期
    ///
    /// 完整校验用它处理内建名称分支；比较借用静态元数据，不通过构造器创建临时布局。
    /// 不修正元数据或样本。参见 [`Self::from_name`] 与 [`Self::is_valid_contract_layout`]。
    pub fn is_canonical_named_layout(&self) -> bool {
        named_layout_contract(&self.name)
            .map(|(channel_count, speakers)| {
                channel_count == self.channel_count && speakers == self.speakers.as_slice()
            })
            .unwrap_or_default()
    }

    /// 验证布局是否仅以规范名称声明正数量、且没有扬声器位置语义。
    ///
    /// # 返回值与约束
    ///
    /// 只有位置为空、数量大于零、名称十进制后缀精确匹配数量时返回真。
    /// 后缀不得有符号、前导零或额外字符；名称比较借用现有字节，不构造临时字符串。
    /// 真值不证明样本长度或后端能力，非空位置列表也不能以离散名称绕过显式位置契约。
    ///
    /// # 生命周期与参考
    ///
    /// 只读取布局，不规范化外部输入。参见 [`Self::discrete`] 与 [`Self::is_valid_contract_layout`]。
    pub fn is_canonical_discrete_layout(&self) -> bool {
        self.speakers.is_empty()
            && self.channel_count > 0
            && canonical_discrete_name_matches(&self.name, self.channel_count)
    }

    // TODO: [CR-FRAMEWORK-AUDIO-0001] 确认反序列化的设备或资产布局在进入后端前调用此校验；当前生产调用点未发现，测试仅验证校验器自身。
    // 现行边界已覆盖此问题：SoundConfig::from_plugin_options 校验或回退包选项，输出设备与外部声源在插件描述符校验后才提交；WAV 由已核对的掩码构造布局。
    /// 验证跨资产与声音服务边界交换的完整布局契约，供外部构造或反序列化后的接收方使用。
    ///
    /// # 返回值
    ///
    /// 有效布局返回真：数量为正，名称非空且无两端空白，内建名称有固定数量与位置顺序，
    /// 规范离散名称有匹配数量且位置为空，其他自定义名称有数量匹配且互异的非空位置列表。
    /// 保留名称和 `discrete_` 前缀优先按各自契约检查，不能降级为宽松的自定义名称。
    ///
    /// # 必要约束与生命周期
    ///
    /// 本方法只借用元数据，返回布尔值，不修改或规范化输入，也不输出诊断原因。
    /// 校验后修改公开字段会使此前结论失效；进入后端仍须核对设备能力、采样率及实际帧数据。
    ///
    /// # 参考
    ///
    /// 声音包选项、输出设备及外部声源描述符的接收边界使用本契约；资产解析保持位置映射。
    /// 参见 [`Self::is_canonical_named_layout`]、[`Self::is_canonical_discrete_layout`] 与 [`Self::has_unique_speakers`]。
    /// 验证可交换布局的完整语义，供接受外部或自定义布局的边界使用。
    pub fn is_valid_contract_layout(&self) -> bool {
        if self.channel_count == 0 || self.name.trim() != self.name.as_str() || self.name.is_empty()
        {
            return false;
        }
        // 先处理保留名称，再处理自定义列表；错误的标准排列不能借自定义分支获得有效结果。
        if named_layout_contract(&self.name).is_some() {
            return self.is_canonical_named_layout();
        }
        if self.name.starts_with("discrete_") {
            return self.is_canonical_discrete_layout();
        }
        if self.speakers.is_empty() {
            return self.is_canonical_discrete_layout();
        }
        self.has_matching_speaker_count() && self.has_unique_speakers()
    }
}

// 将离散名称的行为与源码结构检查挂到私有测试模块；性能样本保持独立的忽略测试入口。
#[cfg(test)]
#[path = "channel_layout/tests/allocation_free_discrete_name_tests.rs"]
mod allocation_free_discrete_name_tests;
