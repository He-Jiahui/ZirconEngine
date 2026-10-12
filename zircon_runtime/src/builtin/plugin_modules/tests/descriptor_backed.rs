use super::*;

#[test]
fn descriptor_backed_modules_borrow_owned_text_at_supported_cardinalities() {
    for cardinality in [1, 100, 1_000] {
        let modules = (0..cardinality)
            .map(|index| DescriptorBackedEngineModule {
                descriptor: ModuleDescriptor::new(
                    format!("RuntimePluginModule{index}"),
                    format!("Runtime plugin descriptor {index}"),
                ),
            })
            .collect::<Vec<_>>();

        assert_eq!(modules.len(), cardinality);
        for module in modules {
            assert_eq!(
                module.module_name().as_ptr(),
                module.descriptor.name.as_ptr()
            );
            assert_eq!(
                module.module_description().as_ptr(),
                module.descriptor.description.as_ptr()
            );
        }
    }
}
