use super::{parse_attribute, parse_shape, parse_tensor, OnnxReadError};

#[test]
fn rejects_negative_packed_tensor_dimension() {
    let mut tensor = vec![0x0a, 10];
    tensor.extend(encoded_varint((-1_i64) as u64));
    tensor.extend([0x42, 1, b'x']);

    assert_eq!(
        parse_tensor(&tensor),
        Err(OnnxReadError::InvalidDimension { value: -1 })
    );
}

#[test]
fn rejects_negative_unpacked_tensor_dimension() {
    let mut tensor = vec![0x08];
    tensor.extend(encoded_varint((-1_i64) as u64));
    tensor.extend([0x42, 1, b'x']);

    assert_eq!(
        parse_tensor(&tensor),
        Err(OnnxReadError::InvalidDimension { value: -1 })
    );
}

#[test]
fn rejects_dimension_larger_than_u32() {
    let encoded = encoded_varint(u64::from(u32::MAX) + 1);
    let mut tensor = vec![0x0a, encoded.len() as u8];
    tensor.extend(encoded);
    tensor.extend([0x42, 1, b'x']);

    assert_eq!(
        parse_tensor(&tensor),
        Err(OnnxReadError::InvalidDimension {
            value: i64::from(u32::MAX) + 1,
        })
    );
}

#[test]
fn rejects_negative_value_info_dimension() {
    let dimension = {
        let encoded = encoded_varint((-1_i64) as u64);
        let mut bytes = vec![0x08];
        bytes.extend(encoded);
        bytes
    };
    let mut shape = vec![0x0a, dimension.len() as u8];
    shape.extend(dimension);

    assert_eq!(
        parse_shape(&shape),
        Err(OnnxReadError::InvalidDimension { value: -1 })
    );
}

#[test]
fn rejects_invalid_utf8_tensor_name() {
    assert_eq!(
        parse_tensor(&[0x42, 1, 0xff]),
        Err(OnnxReadError::InvalidUtf8String)
    );
}

#[test]
fn rejects_invalid_utf8_attribute_string() {
    assert_eq!(
        parse_attribute(&[0x22, 1, 0xff]),
        Err(OnnxReadError::InvalidUtf8String)
    );
}

fn encoded_varint(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            return bytes;
        }
    }
}
