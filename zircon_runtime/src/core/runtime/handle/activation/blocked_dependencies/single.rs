use super::super::super::super::descriptors::RegistryName;

pub(in super::super) fn dependency_slice_contains_service(
    dependencies: &[RegistryName],
    service_name: &RegistryName,
) -> bool {
    match dependencies {
        [] => false,
        [dependency] => dependency == service_name,
        [first_dependency, second_dependency] => {
            first_dependency == service_name || second_dependency == service_name
        }
        [first_dependency, second_dependency, third_dependency] => {
            first_dependency == service_name
                || second_dependency == service_name
                || third_dependency == service_name
        }
        [
            first_dependency,
            second_dependency,
            third_dependency,
            fourth_dependency,
        ] => {
            first_dependency == service_name
                || second_dependency == service_name
                || third_dependency == service_name
                || fourth_dependency == service_name
        }
        [
            first_dependency,
            second_dependency,
            third_dependency,
            fourth_dependency,
            fifth_dependency,
        ] => {
            first_dependency == service_name
                || second_dependency == service_name
                || third_dependency == service_name
                || fourth_dependency == service_name
                || fifth_dependency == service_name
        }
        _ => {
            for dependency in dependencies {
                if dependency == service_name {
                    return true;
                }
            }
            false
        }
    }
}
