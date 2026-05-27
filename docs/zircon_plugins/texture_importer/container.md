---
related_code:
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs
  - zircon_plugins/texture_importer/runtime/src/container/astc.rs
  - zircon_plugins/texture_importer/runtime/src/container/dds.rs
  - zircon_plugins/texture_importer/runtime/src/container/ktx.rs
  - zircon_plugins/texture_importer/runtime/src/container/support.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/mod.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/common.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/astc.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/diagnostics.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/settings.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
implementation_files:
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs
  - zircon_plugins/texture_importer/runtime/src/container/astc.rs
  - zircon_plugins/texture_importer/runtime/src/container/dds.rs
  - zircon_plugins/texture_importer/runtime/src/container/ktx.rs
  - zircon_plugins/texture_importer/runtime/src/container/support.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/mod.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/common.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/astc.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/diagnostics.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/settings.rs
plan_sources:
  - .codex/plans/Asset Importer 插件化补齐计划.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
tests:
  - zircon_plugins/texture_importer/runtime/src/container/tests/astc.rs::astc_container_importer_reads_block_and_size
  - zircon_plugins/texture_importer/runtime/src/container/tests/astc.rs::astc_container_importer_reads_3d_block_and_depth
  - zircon_plugins/texture_importer/runtime/src/container/tests/astc.rs::astc_container_importer_rejects_truncated_3d_block_payload
  - zircon_plugins/texture_importer/runtime/src/container/tests/astc.rs::astc_container_importer_rejects_truncated_block_payload
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_ignores_mip_count_without_mipmap_flag
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_preserves_compressed_payload
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_rejects_declared_zero_mip_count
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_rejects_fourcc_flag_without_fourcc_field
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_rejects_fourcc_without_pixel_format_flag
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_rejects_truncated_compressed_mip_chain_payload
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_rejects_incomplete_cubemap_face_flags
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_rejects_malformed_fourcc_tokens
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_rejects_missing_required_header_flags
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_rejects_missing_texture_caps
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_rejects_mip_count_larger_than_extent_chain
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_container_importer_rejects_volume_headers
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_dx10_container_importer_reads_cubemap_array_layers
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_dx10_container_importer_reads_misc_texturecube_flag
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_dx10_container_importer_rejects_invalid_misc_flags2
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_dx10_container_importer_rejects_truncated_compressed_mip_chain_payload
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_dx10_container_importer_rejects_unsupported_misc_flag_bits
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_dx10_container_importer_rejects_unsupported_resource_dimension
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs::dds_dx10_container_importer_rejects_zero_array_size
  - zircon_plugins/texture_importer/runtime/src/container/tests/diagnostics.rs::astc_container_importer_rejects_zero_block_or_extent_fields
  - zircon_plugins/texture_importer/runtime/src/container/tests/diagnostics.rs::container_importer_rejects_invalid_ktx_face_counts
  - zircon_plugins/texture_importer/runtime/src/container/tests/diagnostics.rs::container_importer_reports_invalid_header_diagnostics
  - zircon_plugins/texture_importer/runtime/src/container/tests/diagnostics.rs::container_importer_reports_layer_count_overflow_diagnostics
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_3d_container_keeps_depth_separate_from_array_layers
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_reads_complete_mip_chain
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_reads_1d_dimension
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_reads_key_value_metadata_record
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_rejects_3d_texture_without_height
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_rejects_mip_count_larger_than_extent_chain
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_rejects_truncated_first_level_image_size
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_rejects_truncated_first_level_payload
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_rejects_truncated_key_value_metadata_record_payload
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_rejects_truncated_second_level_image_size
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_rejects_truncated_second_level_payload
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_rejects_truncated_key_value_metadata
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_rejects_unaligned_key_value_metadata
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx1.rs::ktx1_container_importer_rejects_zero_key_value_metadata_record_size
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_3d_container_keeps_depth_separate_from_array_layers
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_accepts_known_supercompression_schemes
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_reads_3d_dimension
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_reads_layers_faces_and_mips
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_3d_texture_without_height
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_invalid_data_format_descriptor_total_size
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_invalid_level_uncompressed_byte_lengths
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_level_payload_inside_level_index
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_metadata_range_overlapping_level_payload
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_metadata_ranges_inside_level_index
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_mip_count_larger_than_extent_chain
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_overlapping_metadata_ranges
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_overlapping_level_payload_ranges
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_truncated_dfd_range
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_truncated_key_value_data_range
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_truncated_level_payload_range
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_truncated_supercompression_global_data_range
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_unsupported_supercompression_schemes
  - zircon_plugins/texture_importer/runtime/src/container/tests/ktx2.rs::ktx2_container_importer_rejects_zero_type_size
  - zircon_plugins/texture_importer/runtime/src/container/tests/settings.rs::container_importer_applies_descriptor_settings_without_expanding_payload
  - zircon_plugins/texture_importer/runtime/src/container/tests/settings.rs::container_importer_rejects_array_layout_without_decoded_rgba
doc_type: module-detail
---

# Texture Container Importer

The texture importer plugin owns lightweight header parsing for DDS, KTX1, KTX2, and ASTC container assets. It does not decompress those payloads. The importer emits `TextureAsset::new_container(...)` with the original bytes, a render-facing descriptor, mip count, array-layer count, and a normalized container format token such as `dds/dxgi-98`, `ktx/gl-internal-0x000083f1`, `ktx2/vk-37/supercompression-1`, or `astc/6x6x1`.

The production parser root lives in `container/mod.rs`; DDS-specific parsing lives in `container/dds.rs`, KTX1/KTX2 parsing lives in `container/ktx.rs`, ASTC parsing lives in `container/astc.rs`, and shared byte readers, parse diagnostics, layer-count checks, and container constants live in `container/support.rs`. Its regression fixtures and importer-facing tests live under the module-local `container/tests/` tree. `mod.rs` stays as the format dispatch and shared type boundary; `common.rs` owns shared import fixtures and byte writers; and format-specific test files cover DDS, KTX1, KTX2, ASTC, descriptor settings, and cross-format diagnostics. Keeping the tests in child modules preserves access to parser-private helpers while preventing either the production parser or a single regression file from growing past the large-file threshold as more texture container boundaries are added.

ASTC headers are validated before descriptor creation. The block dimensions and x/y/z extents must all be nonzero; malformed zero fields fail import with parse diagnostics instead of being silently normalized to one. After the header fields are read, the importer computes the declared compressed block payload as `ceil(width / block_x) * ceil(height / block_y) * ceil(depth / block_z) * 16` and requires those bytes to exist after the 16-byte ASTC header. A depth of one remains the valid 2D ASTC path, while depth greater than one or a 3D block depth marks the descriptor as `RenderImageDimension::D3`.

DDS DX10 cubemap arrays and KTX layer/face pairs multiply layer counts with overflow checks before metadata is emitted. DDS `dwFlags` must declare `DDSD_CAPS`, `DDSD_HEIGHT`, `DDSD_WIDTH`, and `DDSD_PIXELFORMAT`, matching the fields the importer reads for descriptor construction. `dwMipMapCount` only contributes to the emitted mip count when `DDSD_MIPMAPCOUNT` is set; without that flag the importer reports one mip level, and a declared zero mip count fails import. A declared DDS mip count must also fit the parsed 2D extent's natural chain down to 1x1; over-large mip counts fail before payload sizing. DDS `dwCaps` must include `DDSCAPS_TEXTURE`; files missing the basic texture capability fail import before descriptor creation. DDS pixel format flags must agree with the FourCC field: nonzero FourCC bytes require `DDPF_FOURCC`, and `DDPF_FOURCC` requires a non-empty FourCC field. For recognized block-compressed legacy DDS FourCCs and recognized block-compressed DDS DX10 `dxgiFormat` values, the importer walks every declared mip level, computes each level's block payload from `ceil(mip_width / 4) * ceil(mip_height / 4) * layer_count * bytes_per_block`, sums the mip chain, and requires those bytes after the 128-byte DDS header or 148-byte DDS DX10 header before emitting a container asset. DDS DX10 `arraySize` must be nonzero; a malformed zero value fails import instead of being normalized to one, so broken container headers do not produce misleading array metadata. Legacy DDS volume headers are also rejected before descriptor creation when `dwDepth` is nonzero or `DDSCAPS2_VOLUME` is set, because the current DDS path only has stable 2D, array, and cubemap descriptor contracts. DDS DX10 `resourceDimension` is currently accepted only for the texture-2D path that this importer can describe; unknown, zero, 1D, and 3D resource dimensions fail import until those descriptor paths are implemented. DDS headers that set the legacy cubemap bit must also include all six face bits in `caps2`; incomplete cubemap declarations fail import instead of being treated as a complete six-face array. Nonzero DDS FourCC fields must be printable ASCII without embedded NUL bytes, so malformed tokens fail import before they can become lossy `dds/...` format strings. DX10 headers may also identify cubemaps through `miscFlag` `DDS_RESOURCE_MISC_TEXTURECUBE`; in that path, `arraySize` is interpreted as a cube count and the importer reports `arraySize * 6` array layers. Any other `miscFlag` bit is rejected because the importer has no descriptor contract for those options. DDS DX10 `miscFlags2` accepts only the documented alpha modes in the low three bits and requires all reserved upper bits to be zero. KTX face counts are constrained to ordinary textures or cubemaps: `0` and `1` are treated as one face, `6` is treated as a cubemap, and any other value fails import with a parse diagnostic. KTX1 and KTX2 declared mip counts must also fit the parsed width/height/depth natural mip chain down to 1x1x1; impossible KTX mip counts fail before metadata or level payload ranges are walked. KTX1 and KTX2 3D containers keep native depth in `depth_or_array_layers` and force `array_layer_count` to one, so 3D depth is not confused with a 2D array layer count. KTX1/KTX2 headers that declare `depth > 0` must also declare a nonzero height; malformed 3D extents fail import instead of being normalized from an incomplete 1D-style header.

KTX1 key/value metadata is not interpreted by the importer yet, but the declared byte range is still validated. The `bytesOfKeyValueData` field must be a multiple of four bytes and must fit inside the source bytes immediately after the 64-byte KTX1 header; unaligned or truncated files fail import before a descriptor or container payload is emitted. Inside that declared range, the importer walks each `keyAndValueByteSize` record, requires the size field, rejects zero-length records, requires the declared record payload to stay inside `bytesOfKeyValueData`, and advances over each record's four-byte padding. After that metadata range, every declared mip level must contain its `imageSize` field and any nonzero payload bytes. Intermediate levels advance across KTX1's four-byte mip padding before the next level is read, so a header-only file, a malformed metadata record, a truncated first level, or a truncated later mip level cannot masquerade as a usable container payload.

KTX2 headers also keep basic scalar invariants explicit. `typeSize` must be nonzero before the importer reads extent, layer, face, and level-index fields, so malformed headers cannot proceed into descriptor creation with ambiguous element sizing.

KTX2 `supercompressionScheme` is validated before descriptor creation. The importer preserves official Khronos schemes `0` through `3` in the normalized format token, covering no supercompression, BasisLZ, Zstandard, and ZLIB. Reserved values and registered vendor/private schemes are rejected until Zircon has an explicit decoder or fallback contract for them, which keeps unknown bitstreams from entering the GPU-ready path as ordinary containers.

KTX2 metadata ranges are checked before the importer emits a texture asset. The importer currently keeps the original container bytes instead of decoding the data format descriptor, key/value data, or supercompression global data. The header-declared `dfdByteOffset + dfdByteLength`, `kvdByteOffset + kvdByteLength`, and `sgdByteOffset + sgdByteLength` ranges must fit inside the source file so truncated KTX2 metadata cannot masquerade as a valid container payload. Non-empty DFD, KVD, and SGD ranges must also start after the header and level index table, and no two non-empty metadata ranges may overlap. Metadata ranges also cannot overlap any non-empty level payload range, so a header cannot alias descriptor bytes and texture image bytes to the same source region. Metadata ranges that point back into importer-owned structural bytes or alias another metadata or image block are rejected with targeted parse diagnostics. When a DFD range is present, it must be at least four bytes and its leading `dfdTotalSize` field must equal `dfdByteLength`.

Range validation is centralized in the parser helper path. KTX2 32-bit DFD/KVD fields are converted to the same 64-bit checked-add and overlap-guard path used by SGD ranges, and the level-index pass returns the occupied non-empty payload ranges for metadata alias checks. This keeps overflow handling, short-file diagnostics, metadata overlap rejection, payload overlap rejection, and structural-region rejection consistent across all KTX2 data blocks.

KTX2 level index entries are also checked for both byte ranges and uncompressed length invariants. After the level index table itself is proven present, each entry's `byteOffset + byteLength` payload range must fit inside the source bytes before descriptor creation. Non-empty level payloads must also start after the header and level index table, and no two non-empty level payload ranges may overlap, keeping importer diagnostics aligned with the runtime upload guard that rejects level payloads pointing back into metadata or into each other. `uncompressedByteLength` must equal `byteLength` when `supercompressionScheme` is `0`, must be zero for BasisLZ, and must be nonzero for non-empty Zstandard or ZLIB payloads. Any non-BasisLZ `uncompressedByteLength` must also be divisible by `faceCount * max(1, layerCount)` so later upload or inflate paths can split level payloads by image without guessing.
