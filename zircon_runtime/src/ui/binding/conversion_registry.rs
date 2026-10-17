//! 保存转换描述符与 Rust 函数指针，按槽位和提供者代次解析及执行单次值转换。
//! 注册表不持有模型值；执行只借用输入，转换结果和错误由调用方接收。

use std::collections::BTreeMap;

use thiserror::Error;
use zircon_runtime_interface::ui::{
    binding::{
        UiBindingConversionDescriptor, UiBindingConversionHandle, UiBindingConversionId,
        UiBindingConversionProviderError, UiBindingConversionProviderGeneration,
        UiBindingConversionSlot,
    },
    component::{UiValue, UiValueKind},
};

/// 单次值转换的提供者入口；注册表负责检查载荷种类，提供者负责其领域值约束并保留可识别错误。
///
/// # 参数与返回
/// 输入仅在调用期间借用；返回拥有的值或提供者领域错误。函数指针不捕获闭包环境，
/// 输入借用不进入返回类型；种类检查和错误包装由 [`UiBindingConversionRegistry::execute`] 完成。
pub type UiBindingConversionFunction =
    fn(&UiValue) -> Result<UiValue, UiBindingConversionProviderError>;

/// 注册、句柄解析或执行失败的分类；携带的身份、代次、种类和提供者错误均为拥有型数据。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiBindingConversionRegistryError {
    #[error(
        "binding conversion {id} generation {generation:?} is already registered with a different signature"
    )]
    ProviderGenerationCollision {
        id: UiBindingConversionId,
        generation: UiBindingConversionProviderGeneration,
    },
    #[error(
        "binding conversion {id} provider generation regressed from {current:?} to {requested:?}"
    )]
    ProviderGenerationRegression {
        id: UiBindingConversionId,
        current: UiBindingConversionProviderGeneration,
        requested: UiBindingConversionProviderGeneration,
    },
    #[error("binding conversion registry exceeded its u32 slot capacity")]
    SlotCapacityExceeded,
    #[error("binding conversion handle {handle:?} is stale or unloaded")]
    StaleHandle { handle: UiBindingConversionHandle },
    #[error(
        "binding conversion handle {handle:?} expected {expected_source:?}->{expected_destination:?} but registered {actual_source:?}->{actual_destination:?}"
    )]
    SignatureMismatch {
        handle: UiBindingConversionHandle,
        expected_source: UiValueKind,
        expected_destination: UiValueKind,
        actual_source: UiValueKind,
        actual_destination: UiValueKind,
    },
    #[error(
        "binding conversion handle {handle:?} expected input {expected:?} but received {actual:?}"
    )]
    InputTypeMismatch {
        handle: UiBindingConversionHandle,
        expected: UiValueKind,
        actual: UiValueKind,
    },
    #[error(
        "binding conversion handle {handle:?} promised output {expected:?} but returned {actual:?}"
    )]
    OutputTypeMismatch {
        handle: UiBindingConversionHandle,
        expected: UiValueKind,
        actual: UiValueKind,
    },
    #[error("binding conversion handle {handle:?} provider failed: {error}")]
    ProviderFailed {
        handle: UiBindingConversionHandle,
        error: UiBindingConversionProviderError,
    },
}

/// 一个活动槽位拥有的描述符和实现；替换时两者一同更新，避免签名与回调错配。
#[derive(Clone, Debug)]
struct RegisteredBindingConversion {
    descriptor: UiBindingConversionDescriptor,
    function: UiBindingConversionFunction,
}

/// 为绑定程序持有转换槽位和提供者代次；句柄仅在产生它的注册表内有效。
/// 升级替换会使旧代次失效，卸载不复用槽位，防止已编译绑定误调用另一提供者。
///
/// # 所有权与代次
/// 描述符和函数指针保存在表内；解析返回的引用借用自该表，不能跨越其可变借用。
/// 修订号使用饱和加法，记录注册表变动，不替代提供者代次校验。
#[derive(Clone, Debug, Default)]
pub struct UiBindingConversionRegistry {
    // 空槽保留原索引；新身份始终追加，因此卸载不会把旧槽分给另一个身份。
    slots: Vec<Option<RegisteredBindingConversion>>,
    // 只索引活动身份；register 建立映射，unregister 同时移除映射并清空槽位。
    slots_by_id: BTreeMap<UiBindingConversionId, UiBindingConversionSlot>,
    revision: u64,
}

impl UiBindingConversionRegistry {
    /// 创建没有活动身份、槽位和修订变动的空注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 返回饱和递增的表修订号；新增、升级和卸载推进一次，幂等注册不推进。
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// 返回活动身份数，不包含卸载后保留的空槽。
    pub fn registered_count(&self) -> usize {
        self.slots_by_id.len()
    }

    /// 同一描述符重复注册视为幂等；若实现发生变化，调用者必须提升 provider_generation。
    /// 此契约按描述符判定身份，不比较函数地址，也不会用同代次的新函数替换旧实现。
    ///
    /// # 参数
    /// - `descriptor`：拥有的身份、签名和提供者代次；更高代次替换同一身份的当前槽位。
    /// - `function`：与描述符配套的 Rust 回调；首次注册或升级时保存到表内。
    ///
    /// # 返回与失败
    /// 成功返回当前槽位及代次组成的句柄。代次回退、同代次签名冲突或槽位容量不足
    /// 返回分类错误，均在修改表之前退出；新身份才会追加槽位。
    pub fn register(
        &mut self,
        descriptor: UiBindingConversionDescriptor,
        function: UiBindingConversionFunction,
    ) -> Result<UiBindingConversionHandle, UiBindingConversionRegistryError> {
        if let Some(slot) = self.slots_by_id.get(&descriptor.id).copied() {
            let existing = self.slots[slot.get() as usize]
                .as_ref()
                .expect("active conversion IDs must reference an occupied slot");
            if existing.descriptor == descriptor {
                return Ok(handle_for(slot, &existing.descriptor));
            }
            if descriptor.provider_generation < existing.descriptor.provider_generation {
                return Err(
                    UiBindingConversionRegistryError::ProviderGenerationRegression {
                        id: descriptor.id,
                        current: existing.descriptor.provider_generation,
                        requested: descriptor.provider_generation,
                    },
                );
            }
            if descriptor.provider_generation == existing.descriptor.provider_generation {
                return Err(
                    UiBindingConversionRegistryError::ProviderGenerationCollision {
                        id: descriptor.id,
                        generation: descriptor.provider_generation,
                    },
                );
            }

            let handle = handle_for(slot, &descriptor);
            self.slots[slot.get() as usize] = Some(RegisteredBindingConversion {
                descriptor,
                function,
            });
            self.revision = self.revision.saturating_add(1);
            return Ok(handle);
        }

        let slot_index = u32::try_from(self.slots.len())
            .map_err(|_| UiBindingConversionRegistryError::SlotCapacityExceeded)?;
        let slot = UiBindingConversionSlot::new(slot_index);
        let handle = handle_for(slot, &descriptor);
        self.slots_by_id.insert(descriptor.id.clone(), slot);
        self.slots.push(Some(RegisteredBindingConversion {
            descriptor,
            function,
        }));
        self.revision = self.revision.saturating_add(1);
        Ok(handle)
    }

    /// 解析本表产生的句柄，用于执行前读取当前描述符。
    ///
    /// # 参数与返回
    /// `handle` 必须指向本表的活动槽位及相同代次；越界、空槽或代次不符返回过期错误。
    /// 成功引用借用自 `self`，不复制描述符，也不携带跨注册表的身份保证。
    pub fn resolve(
        &self,
        handle: UiBindingConversionHandle,
    ) -> Result<&UiBindingConversionDescriptor, UiBindingConversionRegistryError> {
        let registered = self
            .slots
            .get(handle.slot().get() as usize)
            .and_then(Option::as_ref)
            .filter(|registered| {
                registered.descriptor.provider_generation == handle.provider_generation()
            })
            .ok_or(UiBindingConversionRegistryError::StaleHandle { handle })?;
        Ok(&registered.descriptor)
    }

    /// 在有效句柄上核对调用方要求的精确输入、输出种类。
    ///
    /// # 参数与返回
    /// `handle` 遵循 [`Self::resolve`] 的作用域；`source`、`destination` 与声明逐项相等
    /// 才返回借用自本表的描述符。此处的 `Any` 也按精确枚举值比较，不作通配。
    pub fn resolve_typed(
        &self,
        handle: UiBindingConversionHandle,
        source: UiValueKind,
        destination: UiValueKind,
    ) -> Result<&UiBindingConversionDescriptor, UiBindingConversionRegistryError> {
        let descriptor = self.resolve(handle)?;
        if descriptor.signature.source != source || descriptor.signature.destination != destination
        {
            return Err(UiBindingConversionRegistryError::SignatureMismatch {
                handle,
                expected_source: source,
                expected_destination: destination,
                actual_source: descriptor.signature.source,
                actual_destination: descriptor.signature.destination,
            });
        }
        Ok(descriptor)
    }

    /// 在当前代次下执行转换，同时核验提供者声明的输入与输出；Any 只放宽值种类，不放宽句柄寿命。
    ///
    /// # 参数与返回
    /// `handle` 选择本表当前活动实现，`input` 仅借用到回调返回；成功值由调用方拥有。
    /// 依次检查句柄、输入种类、提供者结果、输出种类，首个失败即返回相应错误；
    /// 提供者领域错误保留在包装错误中。不会修改注册表或自动写回模型。
    pub fn execute(
        &self,
        handle: UiBindingConversionHandle,
        input: &UiValue,
    ) -> Result<UiValue, UiBindingConversionRegistryError> {
        let registered = self
            .slots
            .get(handle.slot().get() as usize)
            .and_then(Option::as_ref)
            .filter(|registered| {
                registered.descriptor.provider_generation == handle.provider_generation()
            })
            .ok_or(UiBindingConversionRegistryError::StaleHandle { handle })?;
        let signature = registered.descriptor.signature;
        let actual_input = input.kind();
        if !kind_matches(signature.source, actual_input) {
            return Err(UiBindingConversionRegistryError::InputTypeMismatch {
                handle,
                expected: signature.source,
                actual: actual_input,
            });
        }
        let output = (registered.function)(input)
            .map_err(|error| UiBindingConversionRegistryError::ProviderFailed { handle, error })?;
        let actual_output = output.kind();
        if !kind_matches(signature.destination, actual_output) {
            return Err(UiBindingConversionRegistryError::OutputTypeMismatch {
                handle,
                expected: signature.destination,
                actual: actual_output,
            });
        }
        Ok(output)
    }

    /// 卸载句柄对应的活动实现，使该槽位的后续解析和执行失败。
    ///
    /// # 参数与返回
    /// `handle` 必须通过 [`Self::resolve`]；成功返回拥有的旧描述符并推进表修订号。
    /// 失败不修改表；成功后保留空槽，函数指针和身份索引随活动项移除。
    pub fn unregister(
        &mut self,
        handle: UiBindingConversionHandle,
    ) -> Result<UiBindingConversionDescriptor, UiBindingConversionRegistryError> {
        let descriptor = self.resolve(handle)?.clone();
        self.slots[handle.slot().get() as usize] = None;
        self.slots_by_id.remove(&descriptor.id);
        self.revision = self.revision.saturating_add(1);
        Ok(descriptor)
    }
}

/// 执行时的值种类匹配；只有声明侧的 Any 放宽实际载荷种类。
fn kind_matches(expected: UiValueKind, actual: UiValueKind) -> bool {
    expected == UiValueKind::Any || expected == actual
}

/// 从槽位与描述符复制代次构造句柄；描述符借用不进入返回值。
fn handle_for(
    slot: UiBindingConversionSlot,
    descriptor: &UiBindingConversionDescriptor,
) -> UiBindingConversionHandle {
    UiBindingConversionHandle::new(slot, descriptor.provider_generation)
}
