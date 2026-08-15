use super::super::super::super::descriptors::RegistryName;

pub(in super::super) enum TwoServiceDependencyMatch {
    FirstService,
    SecondService,
}

pub(in super::super) fn first_blocked_two_service_dependency(
    dependencies: &[RegistryName],
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
) -> Option<TwoServiceDependencyMatch> {
    match dependencies {
        [] => None,
        [dependency] => {
            if dependency == first_service_name {
                Some(TwoServiceDependencyMatch::FirstService)
            } else if dependency == second_service_name {
                Some(TwoServiceDependencyMatch::SecondService)
            } else {
                None
            }
        }
        [first_dependency, second_dependency] => {
            if first_dependency == first_service_name || second_dependency == first_service_name {
                Some(TwoServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
            {
                Some(TwoServiceDependencyMatch::SecondService)
            } else {
                None
            }
        }
        [first_dependency, second_dependency, third_dependency] => {
            if first_dependency == first_service_name
                || second_dependency == first_service_name
                || third_dependency == first_service_name
            {
                Some(TwoServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
                || third_dependency == second_service_name
            {
                Some(TwoServiceDependencyMatch::SecondService)
            } else {
                None
            }
        }
        [
            first_dependency,
            second_dependency,
            third_dependency,
            fourth_dependency,
        ] => {
            if first_dependency == first_service_name
                || second_dependency == first_service_name
                || third_dependency == first_service_name
                || fourth_dependency == first_service_name
            {
                Some(TwoServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
                || third_dependency == second_service_name
                || fourth_dependency == second_service_name
            {
                Some(TwoServiceDependencyMatch::SecondService)
            } else {
                None
            }
        }
        [
            first_dependency,
            second_dependency,
            third_dependency,
            fourth_dependency,
            fifth_dependency,
        ] => {
            if first_dependency == first_service_name
                || second_dependency == first_service_name
                || third_dependency == first_service_name
                || fourth_dependency == first_service_name
                || fifth_dependency == first_service_name
            {
                Some(TwoServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
                || third_dependency == second_service_name
                || fourth_dependency == second_service_name
                || fifth_dependency == second_service_name
            {
                Some(TwoServiceDependencyMatch::SecondService)
            } else {
                None
            }
        }
        _ => {
            let mut second_service_blocked = false;
            for dependency in dependencies {
                if dependency == first_service_name {
                    return Some(TwoServiceDependencyMatch::FirstService);
                }
                if dependency == second_service_name {
                    second_service_blocked = true;
                }
            }
            if second_service_blocked {
                Some(TwoServiceDependencyMatch::SecondService)
            } else {
                None
            }
        }
    }
}
