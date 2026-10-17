use serde::{Deserialize, Serialize};

use super::{NavMeshAsset, NavMeshBakeReport};

pub const NAVIGATION_BAKE_SCENE_OPERATION: &str = "navigation.bake.scene";
pub const NAVIGATION_BAKE_SURFACE_OPERATION: &str = "navigation.bake.surface";
pub const NAVIGATION_CLEAR_SURFACE_OPERATION: &str = "navigation.bake.clear_surface";
pub const NAVIGATION_RESTORE_BAKE_OPERATION: &str = "navigation.bake.restore_snapshot";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavigationClearBakeRequest {
    pub surface_entity: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 记录某个 surface 的已发布烘焙资源，供清除、恢复及编辑器撤销操作交换前后状态。
pub struct NavigationGeneratedBakeSnapshot {
    pub surface_entity: Option<u64>,
    pub asset: Option<NavMeshAsset>,
    pub output_asset: Option<String>,
}

impl NavigationGeneratedBakeSnapshot {
    pub fn empty(surface_entity: Option<u64>) -> Self {
        Self {
            surface_entity,
            asset: None,
            output_asset: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 一起携带烘焙前后快照与报告，供操作总线和撤销记录还原生成资源。
pub struct NavigationGeneratedBakeChange {
    pub before: NavigationGeneratedBakeSnapshot,
    pub after: NavigationGeneratedBakeSnapshot,
    pub report: Option<NavMeshBakeReport>,
}

pub const NAVIGATION_HISTORY_BAKE_SCENE_V1: &str = "navigation.history.v1.bake.scene";
pub const NAVIGATION_HISTORY_BAKE_SURFACE_V1: &str = "navigation.history.v1.bake.surface";
pub const NAVIGATION_HISTORY_CLEAR_SURFACE_V1: &str = "navigation.history.v1.clear.surface";
pub const NAVIGATION_HISTORY_RESTORE_V1: &str = "navigation.history.v1.restore";
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct NavigationHistoryBudgetV1 {
    pub version: u32,
    pub retained_bytes: u64,
    pub peak_bytes: u64,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct NavigationHistoryRequestV1 {
    pub budget: NavigationHistoryBudgetV1,
    pub payload: serde_json::Value,
}

impl NavigationHistoryBudgetV1 {
    /// Proves the actual normal JSON transport and the borrowed typed owners
    /// before publication. No decoding or cloning precedes this bound.
    pub fn check_snapshots_v1(
        &self,
        result: &serde_json::Value,
        before: &NavigationGeneratedBakeSnapshot,
        after: &NavigationGeneratedBakeSnapshot,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        use zircon_runtime_interface::retained_memory::*;
        if self.version != 1 || self.retained_bytes == 0 || self.peak_bytes < self.retained_bytes {
            return Err(RetentionContractError::Provider {
                reason: "invalid navigation history limits",
            });
        }
        let model = verified_std_model_v1()?;
        let mut json =
            RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(self.peak_bytes));
        json.owned(result)?;
        let mut projection = NavigationJsonProjectionV1::new(model, self.peak_bytes);
        projection
            .child(&navigation_history_wire_result_v1(result))
            .map_err(|_| RetentionContractError::Provider {
                reason: "navigation result transport exceeds its finite projection",
            })?;
        json.array::<u8>(
            projection
                .wire_capacity()
                .map_err(|_| RetentionContractError::Overflow)?,
        )?;
        let mut typed =
            RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(self.retained_bytes));
        typed.owned(before)?;
        typed.owned(after)?;
        // The fixed request/result/transport/decode steps may coexist with
        // both typed snapshots. The versioned protocol reserves eight owners.
        let peak = json.bytes().checked_add(typed.bytes())?.checked_mul(8)?;
        if peak.get() > self.peak_bytes {
            return Err(RetentionContractError::Budget {
                budget: "navigation history DTO peak",
                requested: peak.get(),
                limit: self.peak_bytes,
            });
        }
        Ok(())
    }
}

/// Fixed header backing is separate from the original command/request owner.
/// Its keys, field counts and longest wire ID are closed by this v1 protocol.
pub fn navigation_history_envelope_peak_v1(
    model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
) -> Result<u64, zircon_runtime_interface::retained_memory::RetentionContractError> {
    use zircon_runtime_interface::retained_memory::*;
    let mut m = RetainedBoundBuilderV1::new(model);
    m.layout(
        std::mem::size_of::<zircon_runtime_interface::ZrRuntimeOperationSubmitRequestV1>(),
        std::mem::align_of::<zircon_runtime_interface::ZrRuntimeOperationSubmitRequestV1>(),
    )?;
    m.layout(
        std::mem::size_of::<serde_json::Value>(),
        std::mem::align_of::<serde_json::Value>(),
    )?;
    m.btree::<String, serde_json::Value>(2)?;
    m.btree::<String, serde_json::Value>(3)?;
    m.btree::<String, serde_json::Value>(2)?;
    m.btree::<String, serde_json::Value>(3)?;
    m.btree::<String, serde_json::Value>(1)?;
    for key in [
        "budget",
        "payload",
        "version",
        "retained_bytes",
        "peak_bytes",
        "_history_retention_v1",
        "version",
        "accepted",
        "_history_retention_v1",
        "version",
        "accepted",
        "reason",
        "prepublication_admission",
    ] {
        m.array::<u8>(key.len())?;
    }
    let wire_id = [
        NAVIGATION_HISTORY_BAKE_SCENE_V1,
        NAVIGATION_HISTORY_BAKE_SURFACE_V1,
        NAVIGATION_HISTORY_CLEAR_SURFACE_V1,
        NAVIGATION_HISTORY_RESTORE_V1,
    ]
    .iter()
    .map(|id| id.len())
    .max()
    .expect("the v1 operation family is nonempty");
    m.array::<u8>(wire_id)?;
    Ok(m.bytes().checked_mul(8)?.get())
}

#[derive(Debug)]
struct NavigationJsonProjectionErrorV1;
impl std::fmt::Display for NavigationJsonProjectionErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("navigation JSON projection exceeds its admitted owner model")
    }
}
impl std::error::Error for NavigationJsonProjectionErrorV1 {}
impl serde::ser::Error for NavigationJsonProjectionErrorV1 {
    fn custom<T: std::fmt::Display>(_: T) -> Self {
        Self
    }
}
struct NavigationJsonProjectionV1 {
    bound: zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    depth: usize,
    work: usize,
    wire_bytes: u64,
    wire_limit: u64,
}
impl NavigationJsonProjectionV1 {
    fn new(
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Self {
        Self {
            bound: zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1::with_limit(
                model,
                zircon_runtime_interface::retained_memory::RetainedBytesV1::new(limit),
            ),
            depth: 0,
            work: 1_048_576,
            wire_bytes: 0,
            wire_limit: limit,
        }
    }
    fn wire(&mut self, bytes: u64) -> Result<(), NavigationJsonProjectionErrorV1> {
        self.wire_bytes = self
            .wire_bytes
            .checked_add(bytes)
            .ok_or(NavigationJsonProjectionErrorV1)?;
        if self.wire_bytes > self.wire_limit {
            return Err(NavigationJsonProjectionErrorV1);
        }
        Ok(())
    }
    fn wire_capacity(&self) -> Result<usize, NavigationJsonProjectionErrorV1> {
        // serde_json 1.0.150/151 to_vec starts at 128. The admitted std Vec
        // growth doubles or requests the next write, so max(128, 2*encoded)
        // bounds its requested backing. This is a separate allocation, not
        // a substitute for Value's unescaped strings, arrays or map nodes.
        usize::try_from(
            self.wire_bytes
                .checked_mul(2)
                .ok_or(NavigationJsonProjectionErrorV1)?
                .max(128),
        )
        .map_err(|_| NavigationJsonProjectionErrorV1)
    }
    fn finish(
        mut self,
    ) -> Result<
        zircon_runtime_interface::retained_memory::RetainedBytesV1,
        NavigationJsonProjectionErrorV1,
    > {
        let wire_capacity = self.wire_capacity()?;
        self.bound
            .array::<u8>(wire_capacity)
            .map_err(|_| NavigationJsonProjectionErrorV1)?;
        Ok(self.bound.bytes())
    }
    fn child<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), NavigationJsonProjectionErrorV1> {
        if self.depth >= 64 || self.work == 0 {
            return Err(NavigationJsonProjectionErrorV1);
        }
        self.work -= 1;
        self.depth += 1;
        let result = value.serialize(&mut *self);
        self.depth -= 1;
        result
    }
    fn string(&mut self, value: &str) -> Result<(), NavigationJsonProjectionErrorV1> {
        self.bound
            .array::<u8>(value.len())
            .map_err(|_| NavigationJsonProjectionErrorV1)?;
        self.wire(
            (value.len() as u64)
                .checked_mul(6)
                .and_then(|n| n.checked_add(2))
                .ok_or(NavigationJsonProjectionErrorV1)?,
        )
    }
    fn sequence(&mut self, len: usize) -> Result<(), NavigationJsonProjectionErrorV1> {
        if len > self.work {
            return Err(NavigationJsonProjectionErrorV1);
        }
        self.bound
            .array::<serde_json::Value>(len)
            .map_err(|_| NavigationJsonProjectionErrorV1)?;
        self.wire(
            (len as u64)
                .checked_mul(2)
                .and_then(|n| n.checked_add(2))
                .ok_or(NavigationJsonProjectionErrorV1)?,
        )
    }
    fn object(&mut self, len: usize) -> Result<(), NavigationJsonProjectionErrorV1> {
        if len > self.work {
            return Err(NavigationJsonProjectionErrorV1);
        }
        self.bound
            .btree::<String, serde_json::Value>(len)
            .map_err(|_| NavigationJsonProjectionErrorV1)?;
        self.wire(
            (len as u64)
                .checked_mul(2)
                .and_then(|n| n.checked_add(2))
                .ok_or(NavigationJsonProjectionErrorV1)?,
        )
    }
}
macro_rules! navigation_json_scalar_v1 {
    ($($name:ident : $ty:ty => $bytes:expr),* $(,)?) => {$(
        fn $name(self, _: $ty) -> Result<(), Self::Error> { self.wire($bytes) }
    )*};
}
impl<'a> serde::Serializer for &'a mut NavigationJsonProjectionV1 {
    type Ok = ();
    type Error = NavigationJsonProjectionErrorV1;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    navigation_json_scalar_v1!(serialize_bool:bool=>5,serialize_i8:i8=>4,serialize_i16:i16=>6,
        serialize_i32:i32=>11,serialize_i64:i64=>20,serialize_i128:i128=>40,serialize_u8:u8=>3,
        serialize_u16:u16=>5,serialize_u32:u32=>10,serialize_u64:u64=>20,serialize_u128:u128=>39,
        serialize_f32:f32=>24,serialize_f64:f64=>24);
    fn serialize_char(self, v: char) -> Result<(), Self::Error> {
        let mut bytes = [0; 4];
        self.string(v.encode_utf8(&mut bytes))
    }
    fn serialize_str(self, v: &str) -> Result<(), Self::Error> {
        self.string(v)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Self::Error> {
        self.sequence(v.len())?;
        self.wire(
            (v.len() as u64)
                .checked_mul(3)
                .ok_or(NavigationJsonProjectionErrorV1)?,
        )
    }
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.wire(4)
    }
    fn serialize_some<T: serde::Serialize + ?Sized>(self, v: &T) -> Result<(), Self::Error> {
        self.child(v)
    }
    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.wire(4)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Self::Error> {
        self.wire(4)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        v: &'static str,
    ) -> Result<(), Self::Error> {
        self.string(v)
    }
    fn serialize_newtype_struct<T: serde::Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        self.child(v)
    }
    fn serialize_newtype_variant<T: serde::Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        key: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        self.object(1)?;
        self.string(key)?;
        self.child(v)
    }
    fn serialize_seq(self, n: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.sequence(n.ok_or(NavigationJsonProjectionErrorV1)?)?;
        Ok(self)
    }
    fn serialize_tuple(self, n: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.sequence(n)?;
        Ok(self)
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        n: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.sequence(n)?;
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        key: &'static str,
        n: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.object(1)?;
        self.string(key)?;
        self.sequence(n)?;
        Ok(self)
    }
    fn serialize_map(self, n: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.object(n.ok_or(NavigationJsonProjectionErrorV1)?)?;
        Ok(self)
    }
    fn serialize_struct(
        self,
        _: &'static str,
        n: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.object(n)?;
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        key: &'static str,
        n: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.object(1)?;
        self.string(key)?;
        self.object(n)?;
        Ok(self)
    }
    fn collect_str<T: std::fmt::Display + ?Sized>(self, _: &T) -> Result<(), Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
}
impl serde::ser::SerializeSeq for &mut NavigationJsonProjectionV1 {
    type Ok = ();
    type Error = NavigationJsonProjectionErrorV1;
    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        v: &T,
    ) -> Result<(), Self::Error> {
        self.child(v)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl serde::ser::SerializeTuple for &mut NavigationJsonProjectionV1 {
    type Ok = ();
    type Error = NavigationJsonProjectionErrorV1;
    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        v: &T,
    ) -> Result<(), Self::Error> {
        self.child(v)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl serde::ser::SerializeTupleStruct for &mut NavigationJsonProjectionV1 {
    type Ok = ();
    type Error = NavigationJsonProjectionErrorV1;
    fn serialize_field<T: serde::Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> {
        self.child(v)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl serde::ser::SerializeTupleVariant for &mut NavigationJsonProjectionV1 {
    type Ok = ();
    type Error = NavigationJsonProjectionErrorV1;
    fn serialize_field<T: serde::Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> {
        self.child(v)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl serde::ser::SerializeStruct for &mut NavigationJsonProjectionV1 {
    type Ok = ();
    type Error = NavigationJsonProjectionErrorV1;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        k: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        self.string(k)?;
        self.child(v)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl serde::ser::SerializeStructVariant for &mut NavigationJsonProjectionV1 {
    type Ok = ();
    type Error = NavigationJsonProjectionErrorV1;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        k: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        self.string(k)?;
        self.child(v)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl serde::ser::SerializeMap for &mut NavigationJsonProjectionV1 {
    type Ok = ();
    type Error = NavigationJsonProjectionErrorV1;
    fn serialize_key<T: serde::Serialize + ?Sized>(&mut self, k: &T) -> Result<(), Self::Error> {
        k.serialize(NavigationJsonKeyV1(self))
    }
    fn serialize_value<T: serde::Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> {
        self.child(v)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
struct NavigationJsonKeyV1<'a>(&'a mut NavigationJsonProjectionV1);
macro_rules! navigation_json_key_number_v1 {
    ($($name:ident:$ty:ty => $bytes:expr),* $(,)?) => {$(
        fn $name(self,_:$ty)->Result<(),Self::Error>{self.0.bound.array::<u8>($bytes).map_err(|_|NavigationJsonProjectionErrorV1)?;self.0.wire(($bytes as u64)+2)}
    )*};
}
impl serde::Serializer for NavigationJsonKeyV1<'_> {
    type Ok = ();
    type Error = NavigationJsonProjectionErrorV1;
    type SerializeSeq = serde::ser::Impossible<(), Self::Error>;
    type SerializeTuple = serde::ser::Impossible<(), Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<(), Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<(), Self::Error>;
    type SerializeMap = serde::ser::Impossible<(), Self::Error>;
    type SerializeStruct = serde::ser::Impossible<(), Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<(), Self::Error>;
    navigation_json_key_number_v1!(serialize_bool:bool=>5,serialize_i8:i8=>4,serialize_i16:i16=>6,
        serialize_i32:i32=>11,serialize_i64:i64=>20,serialize_i128:i128=>40,
        serialize_u8:u8=>3,serialize_u16:u16=>5,serialize_u32:u32=>10,
        serialize_u64:u64=>20,serialize_u128:u128=>39);
    fn serialize_f32(self, _: f32) -> Result<(), Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_f64(self, _: f64) -> Result<(), Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_char(self, v: char) -> Result<(), Self::Error> {
        let mut bytes = [0; 4];
        self.0.string(v.encode_utf8(&mut bytes))
    }
    fn serialize_str(self, v: &str) -> Result<(), Self::Error> {
        self.0.string(v)
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_none(self) -> Result<(), Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_some<T: serde::Serialize + ?Sized>(self, v: &T) -> Result<(), Self::Error> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        v: &'static str,
    ) -> Result<(), Self::Error> {
        self.0.string(v)
    }
    fn serialize_newtype_struct<T: serde::Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: serde::Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
    fn collect_str<T: std::fmt::Display + ?Sized>(self, _: &T) -> Result<(), Self::Error> {
        Err(NavigationJsonProjectionErrorV1)
    }
}

#[derive(serde::Serialize)]
struct NavigationHistoryWireRequestV1<'a, T: ?Sized> {
    abi_version: u32,
    operation_id: &'static str,
    payload: NavigationHistoryBorrowedRequestV1<'a, T>,
}
#[derive(serde::Serialize)]
struct NavigationHistoryBorrowedRequestV1<'a, T: ?Sized> {
    budget: NavigationHistoryBudgetV1,
    payload: &'a T,
}
#[derive(serde::Serialize)]
struct NavigationHistoryWireResultV1<'a, T: ?Sized> {
    abi_version: u32,
    handle: zircon_runtime_interface::ZrRuntimeOperationHandle,
    operation_id: &'static str,
    outcome: NavigationHistoryBorrowedOutcomeV1<'a, T>,
}
#[derive(serde::Serialize)]
struct NavigationHistoryBorrowedOutcomeV1<'a, T: ?Sized> {
    status: &'static str,
    output: &'a T,
}
fn navigation_history_wire_result_v1<T: ?Sized>(
    output: &T,
) -> NavigationHistoryWireResultV1<'_, T> {
    NavigationHistoryWireResultV1 {
        abi_version: u32::MAX,
        handle: zircon_runtime_interface::ZrRuntimeOperationHandle::invalid(),
        operation_id: NAVIGATION_HISTORY_CLEAR_SURFACE_V1,
        outcome: NavigationHistoryBorrowedOutcomeV1 {
            status: "succeeded",
            output,
        },
    }
}
fn navigation_history_input_peak_v1<T: serde::Serialize + ?Sized>(
    model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
    payload: &T,
    limit: u64,
) -> Result<u64, zircon_runtime_interface::retained_memory::RetentionContractError> {
    use zircon_runtime_interface::retained_memory::*;
    let mut projection = NavigationJsonProjectionV1::new(model, limit);
    projection.bound.layout(
        std::mem::size_of::<serde_json::Value>(),
        std::mem::align_of::<serde_json::Value>(),
    )?;
    projection
        .child(&NavigationHistoryWireRequestV1 {
            abi_version: u32::MAX,
            operation_id: NAVIGATION_HISTORY_CLEAR_SURFACE_V1,
            payload: NavigationHistoryBorrowedRequestV1 {
                budget: NavigationHistoryBudgetV1 {
                    version: 1,
                    retained_bytes: u64::MAX,
                    peak_bytes: u64::MAX,
                },
                payload,
            },
        })
        .map_err(|_| RetentionContractError::Budget {
            budget: "navigation request projection",
            requested: u64::MAX,
            limit,
        })?;
    Ok(projection
        .finish()
        .map_err(|_| RetentionContractError::Budget {
            budget: "navigation request transport backing",
            requested: u64::MAX,
            limit,
        })?
        .checked_mul(8)?
        .get())
}
/// Pure pre-allocation quote for the one supported request Value representation.
pub fn navigation_history_request_peak_v1(
    model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
    request: &zircon_runtime_interface::ZrRuntimeOperationSubmitRequestV1,
    limit: u64,
) -> Result<u64, zircon_runtime_interface::retained_memory::RetentionContractError> {
    navigation_history_input_peak_v1(model, &request.payload, limit)
}
/// Pure quote before serde_json::to_value constructs a restore request.
pub fn navigation_history_restore_peak_v1(
    model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
    snapshot: &NavigationGeneratedBakeSnapshot,
    limit: u64,
) -> Result<u64, zircon_runtime_interface::retained_memory::RetentionContractError> {
    navigation_history_input_peak_v1(model, snapshot, limit)
}

#[derive(serde::Serialize)]
struct NavigationHistoryProjectionResultV1<'a> {
    before: &'a NavigationGeneratedBakeSnapshot,
    after: &'a NavigationGeneratedBakeSnapshot,
    report: Option<&'a NavMeshBakeReport>,
    _history_retention_v1: NavigationHistoryProjectionReceiptV1,
}
#[derive(serde::Serialize)]
struct NavigationHistoryProjectionReceiptV1 {
    version: u32,
    accepted: bool,
}
impl NavigationHistoryBudgetV1 {
    /// Meters the closed normal-Value projection before any handoff clone or
    /// serialization. The serializer owns only a fixed stack and checked counters.
    pub fn check_projection_v1(
        &self,
        before: &NavigationGeneratedBakeSnapshot,
        after: &NavigationGeneratedBakeSnapshot,
        report: Option<&NavMeshBakeReport>,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        use zircon_runtime_interface::retained_memory::*;
        if self.version != 1 || self.retained_bytes == 0 || self.peak_bytes < self.retained_bytes {
            return Err(RetentionContractError::Provider {
                reason: "invalid navigation history limits",
            });
        }
        let model = verified_std_model_v1()?;
        let mut typed =
            RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(self.retained_bytes));
        typed.owned(before)?;
        typed.owned(after)?;
        let mut projection = NavigationJsonProjectionV1::new(model, self.peak_bytes);
        projection.bound.layout(
            std::mem::size_of::<serde_json::Value>(),
            std::mem::align_of::<serde_json::Value>(),
        )?;
        let result = NavigationHistoryProjectionResultV1 {
            before,
            after,
            report,
            _history_retention_v1: NavigationHistoryProjectionReceiptV1 {
                version: 1,
                accepted: true,
            },
        };
        projection
            .child(&navigation_history_wire_result_v1(&result))
            .map_err(|_| RetentionContractError::Provider {
                reason: "navigation JSON projection exceeds its finite model",
            })?;
        let projected = projection
            .finish()
            .map_err(|_| RetentionContractError::Budget {
                budget: "navigation JSON and transport backing",
                requested: u64::MAX,
                limit: self.peak_bytes,
            })?;
        let peak = projected.checked_add(typed.bytes())?.checked_mul(8)?;
        if peak.get() > self.peak_bytes {
            return Err(RetentionContractError::Budget {
                budget: "navigation pre-serialization DTO peak",
                requested: peak.get(),
                limit: self.peak_bytes,
            });
        }
        Ok(())
    }
}
