use super::{
    ASSET_MODULE, BUILTIN_HOST_MODULE_HANDLE_CAPACITY, FOUNDATION_MODULE, MATH_MODULE,
    RENDER_MODULE, SCENE_MODULE,
};

#[test]
fn builtin_host_module_handle_capacity_covers_complete_install() {
    let fixed_modules = [
        FOUNDATION_MODULE,
        ASSET_MODULE,
        SCENE_MODULE,
        RENDER_MODULE,
        MATH_MODULE,
    ];

    assert_eq!(BUILTIN_HOST_MODULE_HANDLE_CAPACITY, fixed_modules.len() + 1);
}
