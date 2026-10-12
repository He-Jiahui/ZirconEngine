use super::{
    MeshShaderVertexAttribute, MeshShaderVertexLayoutContract, ShaderVertexInputScalarKind,
};

#[test]
fn vertex_contract_sorts_attributes_for_bounded_lookup() {
    let contract = MeshShaderVertexLayoutContract::try_new([
        MeshShaderVertexAttribute::new(8, ShaderVertexInputScalarKind::Float),
        MeshShaderVertexAttribute::new(0, ShaderVertexInputScalarKind::Float),
        MeshShaderVertexAttribute::new(3, ShaderVertexInputScalarKind::Uint),
    ])
    .expect("unique attributes");

    assert_eq!(
        contract.scalar_kind_at(3),
        Some(ShaderVertexInputScalarKind::Uint)
    );
    assert_eq!(contract.scalar_kind_at(7), None);
}

#[test]
fn vertex_contract_rejects_duplicate_locations() {
    let error = MeshShaderVertexLayoutContract::try_new([
        MeshShaderVertexAttribute::new(0, ShaderVertexInputScalarKind::Float),
        MeshShaderVertexAttribute::new(0, ShaderVertexInputScalarKind::Uint),
    ])
    .expect_err("one shader location cannot have two vertex attributes");

    assert!(error.contains("@location(0)"), "unexpected error: {error}");
}
