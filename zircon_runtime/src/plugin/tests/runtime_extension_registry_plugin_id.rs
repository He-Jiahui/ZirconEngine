use super::plugin_id_from_module_name;

#[test]
fn plugin_id_from_module_name_borrows_the_prefix() {
    let module_name = String::from("weather.runtime");
    let plugin_id = plugin_id_from_module_name(&module_name)
        .expect("runtime module name should expose its plugin id");

    assert_eq!(plugin_id, "weather");
    assert_eq!(plugin_id.as_ptr(), module_name.as_ptr());
}
