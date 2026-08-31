use std::collections::BTreeMap;
use std::fmt;

mod raw_data;

use raw_data::decode_f32;

use super::{OnnxAttribute, OnnxGraph, OnnxNode, OnnxTensor, OnnxTensorDataType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnnxReadError {
    UnexpectedEnd,
    InvalidVarint,
    UnsupportedWireType(u8),
    MissingGraph,
    MissingTensorName,
    InvalidFloatTensorData,
}

impl fmt::Display for OnnxReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for OnnxReadError {}

pub fn read_onnx_graph(bytes: &[u8]) -> Result<OnnxGraph, OnnxReadError> {
    let mut reader = ProtoReader::new(bytes);
    while let Some((field, wire_type)) = reader.next_field()? {
        if field == 7 && wire_type == LENGTH_DELIMITED {
            return parse_graph(reader.read_bytes()?);
        }
        reader.skip_value(wire_type)?;
    }
    Err(OnnxReadError::MissingGraph)
}

fn parse_graph(bytes: &[u8]) -> Result<OnnxGraph, OnnxReadError> {
    let mut graph = OnnxGraph::default();
    let mut reader = ProtoReader::new(bytes);
    while let Some((field, wire_type)) = reader.next_field()? {
        match (field, wire_type) {
            (1, LENGTH_DELIMITED) => graph.nodes.push(parse_node(reader.read_bytes()?)?),
            (5, LENGTH_DELIMITED) => {
                let tensor = parse_tensor(reader.read_bytes()?)?;
                graph.tensors.insert(tensor.name.clone(), tensor);
            }
            (11 | 12 | 13, LENGTH_DELIMITED) => {
                let (name, shape, data_type) = parse_value_info(reader.read_bytes()?)?;
                if field == 11 {
                    graph.inputs.push(name.clone());
                }
                if field == 12 {
                    graph.outputs.push(name.clone());
                }
                graph
                    .tensors
                    .entry(name.clone())
                    .or_insert_with(|| OnnxTensor {
                        name,
                        shape,
                        data_type,
                        values: None,
                    });
            }
            _ => reader.skip_value(wire_type)?,
        }
    }
    Ok(graph)
}

fn parse_node(bytes: &[u8]) -> Result<OnnxNode, OnnxReadError> {
    let mut node = OnnxNode {
        name: String::new(),
        op_type: String::new(),
        inputs: Vec::new(),
        outputs: Vec::new(),
        attributes: BTreeMap::new(),
    };
    let mut reader = ProtoReader::new(bytes);
    while let Some((field, wire_type)) = reader.next_field()? {
        match (field, wire_type) {
            (1, LENGTH_DELIMITED) => node.inputs.push(read_string(&mut reader)?),
            (2, LENGTH_DELIMITED) => node.outputs.push(read_string(&mut reader)?),
            (3, LENGTH_DELIMITED) => node.name = read_string(&mut reader)?,
            (4, LENGTH_DELIMITED) => node.op_type = read_string(&mut reader)?,
            (5, LENGTH_DELIMITED) => {
                let (name, value) = parse_attribute(reader.read_bytes()?)?;
                if let Some(value) = value {
                    node.attributes.insert(name, value);
                }
            }
            _ => reader.skip_value(wire_type)?,
        }
    }
    if node.name.is_empty() {
        node.name = node.op_type.clone();
    }
    Ok(node)
}

fn parse_attribute(bytes: &[u8]) -> Result<(String, Option<OnnxAttribute>), OnnxReadError> {
    let mut name = String::new();
    let mut value = None;
    let mut reader = ProtoReader::new(bytes);
    while let Some((field, wire_type)) = reader.next_field()? {
        match (field, wire_type) {
            (1, LENGTH_DELIMITED) => name = read_string(&mut reader)?,
            (2, FIXED32) => {
                value = Some(OnnxAttribute::Float(f32::from_bits(reader.read_fixed32()?)))
            }
            (3, VARINT) => value = Some(OnnxAttribute::Int(reader.read_varint()? as i64)),
            (4, LENGTH_DELIMITED) => value = Some(OnnxAttribute::String(read_string(&mut reader)?)),
            (7, LENGTH_DELIMITED) => {
                let mut packed = ProtoReader::new(reader.read_bytes()?);
                let mut values = Vec::new();
                while !packed.is_empty() {
                    values.push(f32::from_bits(packed.read_fixed32()?));
                }
                value = Some(OnnxAttribute::Floats(values));
            }
            (7, FIXED32) => {
                let float = f32::from_bits(reader.read_fixed32()?);
                match &mut value {
                    Some(OnnxAttribute::Floats(values)) => values.push(float),
                    _ => value = Some(OnnxAttribute::Floats(vec![float])),
                }
            }
            (8, LENGTH_DELIMITED) => {
                let mut packed = ProtoReader::new(reader.read_bytes()?);
                let mut values = Vec::new();
                while !packed.is_empty() {
                    values.push(packed.read_varint()? as i64);
                }
                value = Some(OnnxAttribute::Ints(values));
            }
            (8, VARINT) => {
                let integer = reader.read_varint()? as i64;
                match &mut value {
                    Some(OnnxAttribute::Ints(values)) => values.push(integer),
                    _ => value = Some(OnnxAttribute::Ints(vec![integer])),
                }
            }
            _ => reader.skip_value(wire_type)?,
        }
    }
    Ok((name, value))
}

fn parse_tensor(bytes: &[u8]) -> Result<OnnxTensor, OnnxReadError> {
    let mut name = String::new();
    let mut shape = Vec::new();
    let mut data_type = OnnxTensorDataType::Other;
    let mut values = Vec::new();
    let mut raw_data = None;
    let mut reader = ProtoReader::new(bytes);
    while let Some((field, wire_type)) = reader.next_field()? {
        match (field, wire_type) {
            (1, LENGTH_DELIMITED) => read_packed_i64(reader.read_bytes()?, &mut shape)?,
            (1, VARINT) => shape.push(reader.read_varint()? as u32),
            (2, VARINT) => {
                data_type = if reader.read_varint()? == 1 {
                    OnnxTensorDataType::F32
                } else {
                    OnnxTensorDataType::Other
                };
            }
            (4, LENGTH_DELIMITED) => read_packed_f32(reader.read_bytes()?, &mut values)?,
            (4, FIXED32) => values.push(f32::from_bits(reader.read_fixed32()?)),
            (8, LENGTH_DELIMITED) => name = read_string(&mut reader)?,
            (9, LENGTH_DELIMITED) => raw_data = Some(reader.read_bytes()?),
            _ => reader.skip_value(wire_type)?,
        }
    }
    if name.is_empty() {
        return Err(OnnxReadError::MissingTensorName);
    }
    if let Some(raw_data) = raw_data {
        if data_type != OnnxTensorDataType::F32 {
            return Err(OnnxReadError::InvalidFloatTensorData);
        }
        values = decode_f32(raw_data)?;
    }
    Ok(OnnxTensor {
        name,
        shape,
        data_type,
        values: Some(values),
    })
}

fn parse_value_info(bytes: &[u8]) -> Result<(String, Vec<u32>, OnnxTensorDataType), OnnxReadError> {
    let mut name = String::new();
    let mut shape = Vec::new();
    let mut data_type = OnnxTensorDataType::Other;
    let mut reader = ProtoReader::new(bytes);
    while let Some((field, wire_type)) = reader.next_field()? {
        match (field, wire_type) {
            (1, LENGTH_DELIMITED) => name = read_string(&mut reader)?,
            (2, LENGTH_DELIMITED) => (shape, data_type) = parse_type(reader.read_bytes()?)?,
            _ => reader.skip_value(wire_type)?,
        }
    }
    Ok((name, shape, data_type))
}

fn parse_type(bytes: &[u8]) -> Result<(Vec<u32>, OnnxTensorDataType), OnnxReadError> {
    let mut reader = ProtoReader::new(bytes);
    while let Some((field, wire_type)) = reader.next_field()? {
        if field == 1 && wire_type == LENGTH_DELIMITED {
            return parse_tensor_type(reader.read_bytes()?);
        }
        reader.skip_value(wire_type)?;
    }
    Ok((Vec::new(), OnnxTensorDataType::Other))
}

fn parse_tensor_type(bytes: &[u8]) -> Result<(Vec<u32>, OnnxTensorDataType), OnnxReadError> {
    let mut shape = Vec::new();
    let mut data_type = OnnxTensorDataType::Other;
    let mut reader = ProtoReader::new(bytes);
    while let Some((field, wire_type)) = reader.next_field()? {
        match (field, wire_type) {
            (1, VARINT) => {
                data_type = if reader.read_varint()? == 1 {
                    OnnxTensorDataType::F32
                } else {
                    OnnxTensorDataType::Other
                }
            }
            (2, LENGTH_DELIMITED) => shape = parse_shape(reader.read_bytes()?)?,
            _ => reader.skip_value(wire_type)?,
        }
    }
    Ok((shape, data_type))
}

fn parse_shape(bytes: &[u8]) -> Result<Vec<u32>, OnnxReadError> {
    let mut dimensions = Vec::new();
    let mut reader = ProtoReader::new(bytes);
    while let Some((field, wire_type)) = reader.next_field()? {
        if field == 1 && wire_type == LENGTH_DELIMITED {
            let mut dimension_reader = ProtoReader::new(reader.read_bytes()?);
            while let Some((dimension_field, dimension_wire_type)) =
                dimension_reader.next_field()?
            {
                if dimension_field == 1 && dimension_wire_type == VARINT {
                    dimensions.push(dimension_reader.read_varint()? as u32);
                    break;
                }
                dimension_reader.skip_value(dimension_wire_type)?;
            }
        } else {
            reader.skip_value(wire_type)?;
        }
    }
    Ok(dimensions)
}

fn read_packed_i64(bytes: &[u8], output: &mut Vec<u32>) -> Result<(), OnnxReadError> {
    let mut reader = ProtoReader::new(bytes);
    while !reader.is_empty() {
        output.push(reader.read_varint()? as u32);
    }
    Ok(())
}

fn read_packed_f32(bytes: &[u8], output: &mut Vec<f32>) -> Result<(), OnnxReadError> {
    if bytes.len() % 4 != 0 {
        return Err(OnnxReadError::InvalidFloatTensorData);
    }
    output.extend(
        bytes
            .chunks_exact(4)
            .map(|value| f32::from_le_bytes(value.try_into().unwrap())),
    );
    Ok(())
}

fn read_string(reader: &mut ProtoReader<'_>) -> Result<String, OnnxReadError> {
    Ok(String::from_utf8_lossy(reader.read_bytes()?).into_owned())
}

const VARINT: u8 = 0;
const FIXED32: u8 = 5;
const LENGTH_DELIMITED: u8 = 2;

struct ProtoReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> ProtoReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn is_empty(&self) -> bool {
        self.cursor == self.bytes.len()
    }

    fn next_field(&mut self) -> Result<Option<(u32, u8)>, OnnxReadError> {
        if self.is_empty() {
            return Ok(None);
        }
        let key = self.read_varint()?;
        Ok(Some(((key >> 3) as u32, (key & 7) as u8)))
    }

    fn read_varint(&mut self) -> Result<u64, OnnxReadError> {
        let mut value = 0_u64;
        for index in 0..10 {
            let byte = *self
                .bytes
                .get(self.cursor)
                .ok_or(OnnxReadError::UnexpectedEnd)?;
            self.cursor += 1;
            if index == 9 && byte > 1 {
                return Err(OnnxReadError::InvalidVarint);
            }
            let shift = index * 7;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(OnnxReadError::InvalidVarint)
    }

    fn read_fixed32(&mut self) -> Result<u32, OnnxReadError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn read_bytes(&mut self) -> Result<&'a [u8], OnnxReadError> {
        let length =
            usize::try_from(self.read_varint()?).map_err(|_| OnnxReadError::InvalidVarint)?;
        self.take(length)
    }

    fn skip_value(&mut self, wire_type: u8) -> Result<(), OnnxReadError> {
        match wire_type {
            VARINT => self.read_varint().map(|_| ()),
            LENGTH_DELIMITED => self.read_bytes().map(|_| ()),
            FIXED32 => self.take(4).map(|_| ()),
            1 => self.take(8).map(|_| ()),
            other => Err(OnnxReadError::UnsupportedWireType(other)),
        }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], OnnxReadError> {
        let end = self
            .cursor
            .checked_add(length)
            .ok_or(OnnxReadError::UnexpectedEnd)?;
        let bytes = self
            .bytes
            .get(self.cursor..end)
            .ok_or(OnnxReadError::UnexpectedEnd)?;
        self.cursor = end;
        Ok(bytes)
    }
}
