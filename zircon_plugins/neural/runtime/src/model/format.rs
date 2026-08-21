use std::fmt;
use std::mem::size_of;

use crate::ops::{NnOp, NnOpAttrs, NnOpAttrsError, NnOpCode};
use crate::{
    NN_WEIGHT_ALIGNMENT, NnDataType, NnModelAsset, NnModelValidationError, NnTensorDesc,
    NnTensorKind,
};

const ZNN_MAGIC: [u8; 4] = *b"ZRNN";
const ZNN_VERSION: u32 = 1;
const ZNN_HEADER_BYTES: usize = 40;
const ZNN_TENSOR_RECORD_BYTES: usize = 32;
const ZNN_MIN_OP_RECORD_BYTES: usize = 8;
const ZNN_ALLOWED_FLAGS: u32 = 1;
const ZNN_F16_WEIGHTS_FLAG: u32 = 1;
const ZNN_MAX_ARTIFACT_BYTES: usize = 512 * 1024 * 1024;
const ZNN_MAX_WEIGHT_BLOB_BYTES: usize = 512 * 1024 * 1024;
const ZNN_MAX_OP_TABLE_BYTES: usize = 64 * 1024 * 1024;
const ZNN_MAX_TENSOR_COUNT: usize = 1_048_576;
const ZNN_MAX_OP_COUNT: usize = 1_048_576;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NnModelFormatError {
    Validation(NnModelValidationError),
    BadMagic,
    UnsupportedVersion(u32),
    UnsupportedFlags(u32),
    UnexpectedEnd,
    ArithmeticOverflow,
    InvalidTensorDataType(u8),
    InvalidTensorKind(u8),
    InvalidReservedField,
    UnknownOpCode(u16),
    InvalidOpAttrs(NnOpAttrsError),
    InvalidWeightBlobOffset,
    InvalidWeightPrecisionFlag,
    CountTooLarge,
    ResourceLimitExceeded {
        resource: &'static str,
        actual: u64,
        limit: u64,
    },
    DeclaredCountExceedsTableCapacity {
        resource: &'static str,
        declared: usize,
        maximum: usize,
    },
    AllocationFailed {
        resource: &'static str,
        requested_bytes: usize,
    },
}

impl fmt::Display for NnModelFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for NnModelFormatError {}

impl NnModelAsset {
    pub fn to_znn_bytes(&self) -> Result<Vec<u8>, NnModelFormatError> {
        self.validate().map_err(NnModelFormatError::Validation)?;

        let tensor_count =
            u32::try_from(self.tensors.len()).map_err(|_| NnModelFormatError::CountTooLarge)?;
        let op_count =
            u32::try_from(self.ops.len()).map_err(|_| NnModelFormatError::CountTooLarge)?;
        let op_table = encode_ops(&self.ops)?;
        let op_table_size =
            u32::try_from(op_table.len()).map_err(|_| NnModelFormatError::CountTooLarge)?;
        let tensor_table_size = self
            .tensors
            .len()
            .checked_mul(ZNN_TENSOR_RECORD_BYTES)
            .ok_or(NnModelFormatError::ArithmeticOverflow)?;
        let data_end = ZNN_HEADER_BYTES
            .checked_add(tensor_table_size)
            .and_then(|offset| offset.checked_add(op_table.len()))
            .ok_or(NnModelFormatError::ArithmeticOverflow)?;
        let weight_blob_offset = align_up(data_end, NN_WEIGHT_ALIGNMENT as usize)?;
        let weight_blob_size =
            u64::try_from(self.weights.len()).map_err(|_| NnModelFormatError::CountTooLarge)?;

        let mut bytes = Vec::with_capacity(
            weight_blob_offset
                .checked_add(self.weights.len())
                .ok_or(NnModelFormatError::ArithmeticOverflow)?,
        );
        bytes.extend_from_slice(&ZNN_MAGIC);
        write_u32(&mut bytes, ZNN_VERSION);
        write_u32(
            &mut bytes,
            if self.contains_f16_weights() {
                ZNN_F16_WEIGHTS_FLAG
            } else {
                0
            },
        );
        write_u32(&mut bytes, tensor_count);
        write_u32(&mut bytes, op_count);
        write_u32(&mut bytes, op_table_size);
        write_u64(&mut bytes, weight_blob_offset as u64);
        write_u64(&mut bytes, weight_blob_size);
        debug_assert_eq!(bytes.len(), ZNN_HEADER_BYTES);

        for tensor in &self.tensors {
            encode_tensor(&mut bytes, tensor);
        }
        bytes.extend_from_slice(&op_table);
        bytes.resize(weight_blob_offset, 0);
        bytes.extend_from_slice(&self.weights);
        Ok(bytes)
    }

    pub fn from_znn_bytes(bytes: &[u8]) -> Result<Self, NnModelFormatError> {
        enforce_resource_limit("artifact_bytes", bytes.len(), ZNN_MAX_ARTIFACT_BYTES)?;
        let header = bytes
            .get(..ZNN_HEADER_BYTES)
            .ok_or(NnModelFormatError::UnexpectedEnd)?;
        if header[..4] != ZNN_MAGIC {
            return Err(NnModelFormatError::BadMagic);
        }
        let version = read_u32(header, 4)?;
        if version != ZNN_VERSION {
            return Err(NnModelFormatError::UnsupportedVersion(version));
        }
        let flags = read_u32(header, 8)?;
        if flags & !ZNN_ALLOWED_FLAGS != 0 {
            return Err(NnModelFormatError::UnsupportedFlags(flags));
        }

        let tensor_count = usize::try_from(read_u32(header, 12)?)
            .map_err(|_| NnModelFormatError::CountTooLarge)?;
        let op_count = usize::try_from(read_u32(header, 16)?)
            .map_err(|_| NnModelFormatError::CountTooLarge)?;
        let op_table_size = usize::try_from(read_u32(header, 20)?)
            .map_err(|_| NnModelFormatError::CountTooLarge)?;
        let weight_blob_offset = usize::try_from(read_u64(header, 24)?)
            .map_err(|_| NnModelFormatError::CountTooLarge)?;
        let weight_blob_size = usize::try_from(read_u64(header, 32)?)
            .map_err(|_| NnModelFormatError::CountTooLarge)?;
        enforce_resource_limit("tensor_count", tensor_count, ZNN_MAX_TENSOR_COUNT)?;
        enforce_resource_limit("op_count", op_count, ZNN_MAX_OP_COUNT)?;
        enforce_resource_limit("op_table_bytes", op_table_size, ZNN_MAX_OP_TABLE_BYTES)?;
        enforce_resource_limit(
            "weight_blob_bytes",
            weight_blob_size,
            ZNN_MAX_WEIGHT_BLOB_BYTES,
        )?;
        let maximum_ops_in_table = op_table_size / ZNN_MIN_OP_RECORD_BYTES;
        if op_count > maximum_ops_in_table {
            return Err(NnModelFormatError::DeclaredCountExceedsTableCapacity {
                resource: "op_count",
                declared: op_count,
                maximum: maximum_ops_in_table,
            });
        }
        if weight_blob_offset % NN_WEIGHT_ALIGNMENT as usize != 0 {
            return Err(NnModelFormatError::InvalidWeightBlobOffset);
        }

        let tensor_table_end = ZNN_HEADER_BYTES
            .checked_add(
                tensor_count
                    .checked_mul(ZNN_TENSOR_RECORD_BYTES)
                    .ok_or(NnModelFormatError::ArithmeticOverflow)?,
            )
            .ok_or(NnModelFormatError::ArithmeticOverflow)?;
        let op_table_end = tensor_table_end
            .checked_add(op_table_size)
            .ok_or(NnModelFormatError::ArithmeticOverflow)?;
        let weight_blob_end = weight_blob_offset
            .checked_add(weight_blob_size)
            .ok_or(NnModelFormatError::ArithmeticOverflow)?;
        if tensor_table_end > bytes.len()
            || op_table_end > weight_blob_offset
            || weight_blob_end > bytes.len()
        {
            return Err(NnModelFormatError::UnexpectedEnd);
        }

        let mut tensors = Vec::new();
        try_reserve_exact(&mut tensors, tensor_count, "tensor_records")?;
        for record in
            bytes[ZNN_HEADER_BYTES..tensor_table_end].chunks_exact(ZNN_TENSOR_RECORD_BYTES)
        {
            tensors.push(decode_tensor(record)?);
        }
        let ops = decode_ops(&bytes[tensor_table_end..op_table_end], op_count)?;
        let mut weights = Vec::new();
        try_reserve_exact(&mut weights, weight_blob_size, "weight_blob")?;
        weights.extend_from_slice(&bytes[weight_blob_offset..weight_blob_end]);
        let model = Self {
            tensors,
            ops,
            weights,
        };
        let has_f16_weights = flags & ZNN_F16_WEIGHTS_FLAG != 0;
        if model.contains_f16_weights() != has_f16_weights {
            return Err(NnModelFormatError::InvalidWeightPrecisionFlag);
        }
        model.validate().map_err(NnModelFormatError::Validation)?;
        Ok(model)
    }
}

fn encode_tensor(bytes: &mut Vec<u8>, tensor: &NnTensorDesc) {
    bytes.push(tensor.dtype as u8);
    bytes.push(tensor.kind as u8);
    bytes.push(tensor.rank);
    bytes.push(0);
    for dimension in tensor.shape {
        write_u32(bytes, dimension);
    }
    write_u64(bytes, tensor.weight_offset);
    bytes.extend_from_slice(&[0; 4]);
}

fn decode_tensor(record: &[u8]) -> Result<NnTensorDesc, NnModelFormatError> {
    if record[3] != 0 || record[28..32] != [0; 4] {
        return Err(NnModelFormatError::InvalidReservedField);
    }
    let dtype =
        NnDataType::try_from(record[0]).map_err(NnModelFormatError::InvalidTensorDataType)?;
    let kind = NnTensorKind::try_from(record[1]).map_err(NnModelFormatError::InvalidTensorKind)?;
    let shape = std::array::from_fn(|index| read_u32(record, 4 + index * 4).unwrap());
    Ok(NnTensorDesc {
        dtype,
        kind,
        rank: record[2],
        shape,
        weight_offset: read_u64(record, 20)?,
    })
}

fn encode_ops(ops: &[NnOp]) -> Result<Vec<u8>, NnModelFormatError> {
    let mut bytes = Vec::new();
    for op in ops {
        let inputs =
            u8::try_from(op.inputs.len()).map_err(|_| NnModelFormatError::CountTooLarge)?;
        let outputs =
            u8::try_from(op.outputs.len()).map_err(|_| NnModelFormatError::CountTooLarge)?;
        let attrs = op
            .attrs
            .encode(op.code)
            .map_err(NnModelFormatError::InvalidOpAttrs)?;
        let attr_size =
            u16::try_from(attrs.len()).map_err(|_| NnModelFormatError::CountTooLarge)?;
        write_u16(&mut bytes, op.code as u16);
        bytes.push(inputs);
        bytes.push(outputs);
        write_u16(&mut bytes, attr_size);
        write_u16(&mut bytes, 0);
        for tensor in op.inputs.iter().chain(&op.outputs) {
            write_u16(&mut bytes, *tensor);
        }
        bytes.extend_from_slice(&attrs);
        pad_to_four_bytes(&mut bytes);
    }
    Ok(bytes)
}

fn decode_ops(bytes: &[u8], expected_count: usize) -> Result<Vec<NnOp>, NnModelFormatError> {
    let mut cursor = 0;
    let mut ops = Vec::new();
    try_reserve_exact(&mut ops, expected_count, "ops")?;
    while cursor < bytes.len() && ops.len() < expected_count {
        let header_end = cursor
            .checked_add(8)
            .ok_or(NnModelFormatError::ArithmeticOverflow)?;
        let header = bytes
            .get(cursor..header_end)
            .ok_or(NnModelFormatError::UnexpectedEnd)?;
        let code =
            NnOpCode::try_from(read_u16(header, 0)?).map_err(NnModelFormatError::UnknownOpCode)?;
        let input_count = header[2] as usize;
        let output_count = header[3] as usize;
        let attr_size = read_u16(header, 4)? as usize;
        if read_u16(header, 6)? != 0 {
            return Err(NnModelFormatError::InvalidReservedField);
        }
        cursor = header_end;
        let tensor_count = input_count
            .checked_add(output_count)
            .ok_or(NnModelFormatError::ArithmeticOverflow)?;
        let tensor_bytes = tensor_count
            .checked_mul(2)
            .ok_or(NnModelFormatError::ArithmeticOverflow)?;
        let ids_end = cursor
            .checked_add(tensor_bytes)
            .ok_or(NnModelFormatError::ArithmeticOverflow)?;
        let ids = bytes
            .get(cursor..ids_end)
            .ok_or(NnModelFormatError::UnexpectedEnd)?;
        let (inputs, outputs) = decode_tensor_ids(ids, input_count, output_count)?;
        cursor = ids_end;
        let attrs_end = cursor
            .checked_add(attr_size)
            .ok_or(NnModelFormatError::ArithmeticOverflow)?;
        let attrs = bytes
            .get(cursor..attrs_end)
            .ok_or(NnModelFormatError::UnexpectedEnd)?;
        cursor = attrs_end;
        let aligned_cursor = align_up(cursor, 4)?;
        let padding = bytes
            .get(cursor..aligned_cursor)
            .ok_or(NnModelFormatError::UnexpectedEnd)?;
        if padding.iter().any(|byte| *byte != 0) {
            return Err(NnModelFormatError::InvalidReservedField);
        }
        cursor = aligned_cursor;
        ops.push(NnOp::new(
            code,
            inputs,
            outputs,
            NnOpAttrs::decode(code, attrs).map_err(NnModelFormatError::InvalidOpAttrs)?,
        ));
    }
    if cursor != bytes.len() || ops.len() != expected_count {
        return Err(NnModelFormatError::UnexpectedEnd);
    }
    Ok(ops)
}

fn decode_tensor_ids(
    bytes: &[u8],
    input_count: usize,
    output_count: usize,
) -> Result<(Vec<u16>, Vec<u16>), NnModelFormatError> {
    let input_bytes = input_count
        .checked_mul(size_of::<u16>())
        .ok_or(NnModelFormatError::ArithmeticOverflow)?;
    let (encoded_inputs, encoded_outputs) = bytes
        .split_at_checked(input_bytes)
        .ok_or(NnModelFormatError::UnexpectedEnd)?;

    let mut inputs = Vec::new();
    try_reserve_exact(&mut inputs, input_count, "op_inputs")?;
    for encoded in encoded_inputs.chunks_exact(size_of::<u16>()) {
        inputs.push(read_u16(encoded, 0)?);
    }

    let mut outputs = Vec::new();
    try_reserve_exact(&mut outputs, output_count, "op_outputs")?;
    for encoded in encoded_outputs.chunks_exact(size_of::<u16>()) {
        outputs.push(read_u16(encoded, 0)?);
    }
    if inputs.len() != input_count || outputs.len() != output_count {
        return Err(NnModelFormatError::UnexpectedEnd);
    }
    Ok((inputs, outputs))
}

fn enforce_resource_limit(
    resource: &'static str,
    actual: usize,
    limit: usize,
) -> Result<(), NnModelFormatError> {
    if actual > limit {
        return Err(NnModelFormatError::ResourceLimitExceeded {
            resource,
            actual: actual as u64,
            limit: limit as u64,
        });
    }
    Ok(())
}

fn try_reserve_exact<T>(
    values: &mut Vec<T>,
    additional: usize,
    resource: &'static str,
) -> Result<(), NnModelFormatError> {
    values
        .try_reserve_exact(additional)
        .map_err(|_| NnModelFormatError::AllocationFailed {
            resource,
            requested_bytes: additional.saturating_mul(size_of::<T>()),
        })
}

fn pad_to_four_bytes(bytes: &mut Vec<u8>) {
    let target = (bytes.len() + 3) & !3;
    bytes.resize(target, 0);
}

fn align_up(value: usize, alignment: usize) -> Result<usize, NnModelFormatError> {
    value
        .checked_add(alignment - 1)
        .map(|value| value / alignment * alignment)
        .ok_or(NnModelFormatError::ArithmeticOverflow)
}

fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, NnModelFormatError> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or(NnModelFormatError::UnexpectedEnd)?;
    Ok(u16::from_le_bytes(value.try_into().unwrap()))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, NnModelFormatError> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or(NnModelFormatError::UnexpectedEnd)?;
    Ok(u32::from_le_bytes(value.try_into().unwrap()))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, NnModelFormatError> {
    let value = bytes
        .get(offset..offset + 8)
        .ok_or(NnModelFormatError::UnexpectedEnd)?;
    Ok(u64::from_le_bytes(value.try_into().unwrap()))
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::mem::size_of;
    use std::time::Instant;

    use super::{decode_tensor_ids, read_u16};

    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS_PER_SAMPLE: usize = 2_048;
    const INPUT_COUNT: usize = 192;
    const OUTPUT_COUNT: usize = 63;

    #[test]
    #[ignore = "release performance evidence"]
    fn znn_tensor_id_decode_release_gate() {
        let encoded = encoded_tensor_ids(INPUT_COUNT + OUTPUT_COUNT);
        for _ in 0..128 {
            black_box(decode_tensor_ids(&encoded, INPUT_COUNT, OUTPUT_COUNT).unwrap());
            black_box(decode_tensor_ids_legacy(&encoded, INPUT_COUNT));
        }

        let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples_ns.push(measure_legacy(&encoded));
                optimized_samples_ns.push(measure_optimized(&encoded));
            } else {
                optimized_samples_ns.push(measure_optimized(&encoded));
                legacy_samples_ns.push(measure_legacy(&encoded));
            }
        }

        let legacy_p95_ns = nearest_rank_percentile(&legacy_samples_ns, 95);
        let optimized_p95_ns = nearest_rank_percentile(&optimized_samples_ns, 95);
        assert!(
            u128::from(optimized_p95_ns) * 100 <= u128::from(legacy_p95_ns) * 110,
            "optimized P95 {optimized_p95_ns}ns exceeded the 10% regression ceiling over legacy P95 {legacy_p95_ns}ns"
        );

        println!(
            "PERF-MVP-PLUGINS02-ZNN-BOUNDED-LOADER sample_pairs={SAMPLE_PAIRS} iterations_per_sample={ITERATIONS_PER_SAMPLE} tensor_ids_per_op={} legacy_allocations_per_op=3 optimized_allocations_per_op=2 allocation_reduction_pct=33 legacy_id_writes_per_op={} optimized_id_writes_per_op={} id_write_reduction_pct=50 legacy_samples_ns={} optimized_samples_ns={} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} target_ratio_pct=110",
            INPUT_COUNT + OUTPUT_COUNT,
            (INPUT_COUNT + OUTPUT_COUNT) * 2,
            INPUT_COUNT + OUTPUT_COUNT,
            join_samples(&legacy_samples_ns),
            join_samples(&optimized_samples_ns),
        );
    }

    fn measure_legacy(encoded: &[u8]) -> u64 {
        let started = Instant::now();
        for _ in 0..ITERATIONS_PER_SAMPLE {
            black_box(decode_tensor_ids_legacy(encoded, INPUT_COUNT));
        }
        u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn measure_optimized(encoded: &[u8]) -> u64 {
        let started = Instant::now();
        for _ in 0..ITERATIONS_PER_SAMPLE {
            black_box(decode_tensor_ids(encoded, INPUT_COUNT, OUTPUT_COUNT).unwrap());
        }
        u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn decode_tensor_ids_legacy(encoded: &[u8], input_count: usize) -> (Vec<u16>, Vec<u16>) {
        let tensor_ids = encoded
            .chunks_exact(size_of::<u16>())
            .map(|value| read_u16(value, 0).unwrap())
            .collect::<Vec<_>>();
        (
            tensor_ids[..input_count].to_vec(),
            tensor_ids[input_count..].to_vec(),
        )
    }

    fn encoded_tensor_ids(count: usize) -> Vec<u8> {
        (0..count)
            .flat_map(|index| {
                u16::try_from(index)
                    .expect("benchmark tensor id should fit u16")
                    .to_le_bytes()
            })
            .collect()
    }

    fn nearest_rank_percentile(samples: &[u64], percentile: usize) -> u64 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
        ordered[rank.saturating_sub(1)]
    }

    fn join_samples(samples: &[u64]) -> String {
        samples
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
