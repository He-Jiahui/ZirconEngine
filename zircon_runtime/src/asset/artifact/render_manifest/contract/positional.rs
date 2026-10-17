use super::{
    RenderArtifactLayout, RenderArtifactMeshLayout, RenderArtifactTextureLayout,
    RenderSubresourceId,
};
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

// Keep the existing tagged serialization bytes. Bincode cannot deserialize
// serde's internally tagged enums, because they require deserialize_any.
// Its struct representation is a sequence containing the tag and payload.
impl<'de> Deserialize<'de> for RenderSubresourceId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum HumanReadable {
            TextureMipLayer { mip: u32, layer: u32 },
            MeshLod { lod: u16 },
            MeshClusterPage { lod: u16, page: u32 },
        }
        if deserializer.is_human_readable() {
            return Ok(match HumanReadable::deserialize(deserializer)? {
                HumanReadable::TextureMipLayer { mip, layer } => {
                    Self::TextureMipLayer { mip, layer }
                }
                HumanReadable::MeshLod { lod } => Self::MeshLod { lod },
                HumanReadable::MeshClusterPage { lod, page } => Self::MeshClusterPage { lod, page },
            });
        }
        struct Positional;
        impl<'de> Visitor<'de> for Positional {
            type Value = RenderSubresourceId;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a render subresource tag followed by its existing fields")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let tag: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                match tag.as_str() {
                    "texture_mip_layer" => Ok(RenderSubresourceId::TextureMipLayer {
                        mip: seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(1, &self))?,
                        layer: seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(2, &self))?,
                    }),
                    "mesh_lod" => Ok(RenderSubresourceId::MeshLod {
                        lod: seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(1, &self))?,
                    }),
                    "mesh_cluster_page" => Ok(RenderSubresourceId::MeshClusterPage {
                        lod: seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(1, &self))?,
                        page: seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(2, &self))?,
                    }),
                    _ => Err(de::Error::unknown_variant(
                        &tag,
                        &["texture_mip_layer", "mesh_lod", "mesh_cluster_page"],
                    )),
                }
            }
        }
        // Positional encodings do not contain a tuple-length header; the tag
        // selects whether the previously serialized struct has two or three fields.
        deserializer.deserialize_tuple(3, Positional)
    }
}

impl<'de> Deserialize<'de> for RenderArtifactLayout {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum HumanReadable {
            Texture { layout: RenderArtifactTextureLayout },
            Mesh { layout: RenderArtifactMeshLayout },
        }
        if deserializer.is_human_readable() {
            return Ok(match HumanReadable::deserialize(deserializer)? {
                HumanReadable::Texture { layout } => Self::Texture { layout },
                HumanReadable::Mesh { layout } => Self::Mesh { layout },
            });
        }
        struct Positional;
        impl<'de> Visitor<'de> for Positional {
            type Value = RenderArtifactLayout;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a render layout tag followed by its existing payload")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let tag: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                match tag.as_str() {
                    "texture" => Ok(RenderArtifactLayout::Texture {
                        layout: seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(1, &self))?,
                    }),
                    "mesh" => Ok(RenderArtifactLayout::Mesh {
                        layout: seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(1, &self))?,
                    }),
                    _ => Err(de::Error::unknown_variant(&tag, &["texture", "mesh"])),
                }
            }
        }
        deserializer.deserialize_tuple(2, Positional)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::serialization::PositionalEncoding;
    #[test]
    fn existing_tagged_subresource_bytes_decode_without_changing_the_wire_format() {
        let cases = [
            (
                RenderSubresourceId::TextureMipLayer { mip: 3, layer: 7 },
                "texture_mip_layer",
                [3_u32.to_le_bytes(), 7_u32.to_le_bytes()].concat(),
            ),
            (
                RenderSubresourceId::MeshLod { lod: 5 },
                "mesh_lod",
                5_u16.to_le_bytes().to_vec(),
            ),
            (
                RenderSubresourceId::MeshClusterPage { lod: 5, page: 9 },
                "mesh_cluster_page",
                [
                    5_u16.to_le_bytes().as_slice(),
                    9_u32.to_le_bytes().as_slice(),
                ]
                .concat(),
            ),
        ];
        for (value, tag, fields) in cases {
            let mut expected = (tag.len() as u64).to_le_bytes().to_vec();
            expected.extend_from_slice(tag.as_bytes());
            expected.extend_from_slice(&fields);
            assert_eq!(
                PositionalEncoding::FixedLittleEndian
                    .encode_bounded(&value, 256)
                    .unwrap(),
                expected
            );
            assert_eq!(
                PositionalEncoding::FixedLittleEndian
                    .decode::<RenderSubresourceId>(&expected)
                    .unwrap(),
                value
            );
            assert_eq!(
                serde_json::from_slice::<RenderSubresourceId>(&serde_json::to_vec(&value).unwrap())
                    .unwrap(),
                value
            );
        }
    }
}
