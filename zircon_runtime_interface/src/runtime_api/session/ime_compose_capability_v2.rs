//! Negotiated transport selection for versioned IME composition payloads.
//!
//! The fixed `ZrRuntimeEventV1` layout remains the transport envelope.  A composition V2 event
//! uses a new IME state only after both endpoints explicitly advertise this capability.  This
//! keeps an older Runtime API from receiving a state it cannot decode.

//! 本模块只判定双方是否满足 V2 载荷的版本、能力和 schema 门槛。
//! 协商结果供宿主选择线格式；不会安装 Runtime 会话配置或派发事件。

use super::ime_composition_v2::ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA;

/// 对端用于明确声明支持组合 V2 的能力字符串。
pub const ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY: &str = "runtime.ime.composition.v2";
/// 在既有 IME 事件 kind 中标识组合 V2 载荷的状态值。
pub const ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE: u32 = 10;
/// 支持 V2 组合状态所需的最低 Runtime API 版本。
pub const ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 表示协商后供事件发送方使用的线格式选择；本值本身不发送事件。
pub enum ZrRuntimeImeCompositionWireVersion {
    /// 对端条件不足以安全使用组合 V2 时使用旧 IME 线格式。
    LegacyV1,
    /// 对端满足本模块的 API、能力和 schema 检查时使用组合 V2 载荷。
    CompositionV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 保存单个对端的协商输入快照，不持有其配置缓冲区或会话资源。
pub struct ZrRuntimeImeCompositionNegotiation {
    /// 从已验证配置或对端声明记录的 API 版本。
    peer_api_version: u32,
    /// 解析到的组合 schema；缺失时不能通过 V2 门槛。
    peer_schema: Option<u16>,
    /// 是否见到精确的组合 V2 能力字符串。
    peer_capability: bool,
}

impl ZrRuntimeImeCompositionNegotiation {
    /// Builds a negotiation from the bounded configuration ABI after its exact validation.
    /// Keeping this constructor separate from string parsing prevents a V8 table match from
    /// silently implying support for the state-10 wire format.
    /// 直接记录已由调用方验证的标量；本函数不再校验版本、schema 或能力组合。
    /// 返回值拥有这些标量，不借用配置对象，也不执行会话安装。
    pub const fn from_raw(
        peer_api_version: u32,
        peer_schema: Option<u16>,
        peer_capability: bool,
    ) -> Self {
        Self {
            peer_api_version,
            peer_schema,
            peer_capability,
        }
    }

    /// A missing capability is deliberately a V1-only session.  The producer must not infer
    /// support from a matching DLL or from the V8 table shape alone.
    /// 逐项读取借用的能力字符串；无法解析的 schema 项忽略，多个合法值以最后一个为准。
    /// 返回值只保存解析出的标量，不保留输入字符串；裸能力使用本实现 schema 作为默认值。
    pub fn from_peer<I, S>(peer_api_version: u32, peer_capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut advertised_schema = None;
        let mut peer_capability = false;
        for capability in peer_capabilities {
            let capability = capability.as_ref();
            if capability == ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY {
                peer_capability = true;
            } else if let Some(schema) =
                capability.strip_prefix("runtime.ime.composition.v2.schema=")
            {
                if let Ok(schema) = schema.parse::<u16>() {
                    advertised_schema = Some(schema);
                }
            }
        }
        // A bare capability advertises this implementation's schema. If a peer sends an
        // explicit schema, preserve it even when it appears before the capability so a wrong
        // schema cannot be overwritten by iteration order.
        let peer_schema = advertised_schema
            .or_else(|| peer_capability.then_some(ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA));
        Self {
            peer_api_version,
            peer_schema,
            peer_capability,
        }
    }

    /// 构造明确不支持组合 V2 的默认协商状态，用于旧版或未协商对端。
    pub const fn legacy() -> Self {
        Self {
            peer_api_version: 0,
            peer_schema: None,
            peer_capability: false,
        }
    }

    /// 仅当 API 版本、精确能力标记和 schema 同时满足时返回 true。
    pub const fn supports_v2(&self) -> bool {
        self.peer_api_version >= ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION
            && self.peer_capability
            && matches!(self.peer_schema, Some(ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA))
    }

    /// 将当前门槛映射为发送方线格式提示；不配置会话，也不保证运行时接受事件。
    pub const fn wire_version(&self) -> ZrRuntimeImeCompositionWireVersion {
        if self.supports_v2() {
            ZrRuntimeImeCompositionWireVersion::CompositionV2
        } else {
            ZrRuntimeImeCompositionWireVersion::LegacyV1
        }
    }

    /// 返回构造时记录的对端 API 版本。
    pub const fn peer_api_version(&self) -> u32 {
        self.peer_api_version
    }

    /// 返回解析或直接记录的对端 schema；无声明时为 None。
    pub const fn peer_schema(&self) -> Option<u16> {
        self.peer_schema
    }
}

#[cfg(test)]
#[path = "../tests/ime_compose_capability_v2.rs"]
mod tests;
