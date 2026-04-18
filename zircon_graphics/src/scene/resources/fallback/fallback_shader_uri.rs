use zircon_resource::ResourceLocator;

pub(crate) fn fallback_shader_uri() -> ResourceLocator {
    ResourceLocator::parse("builtin://shader/pbr.wgsl").expect("builtin fallback shader uri")
}
