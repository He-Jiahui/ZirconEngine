use super::super::super::super::descriptors::RegistryName;

pub(in super::super) enum ThreeServiceDependencyMatch {
    FirstService,
    SecondService,
    ThirdService,
}

pub(in super::super) fn first_blocked_three_service_dependency(
    dependencies: &[RegistryName],
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
    third_service_name: &RegistryName,
) -> Option<ThreeServiceDependencyMatch> {
    match dependencies {
        [] => None,
        [dependency] => {
            if dependency == first_service_name {
                Some(ThreeServiceDependencyMatch::FirstService)
            } else if dependency == second_service_name {
                Some(ThreeServiceDependencyMatch::SecondService)
            } else if dependency == third_service_name {
                Some(ThreeServiceDependencyMatch::ThirdService)
            } else {
                None
            }
        }
        [first_dependency, second_dependency] => {
            if first_dependency == first_service_name || second_dependency == first_service_name {
                Some(ThreeServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
            {
                Some(ThreeServiceDependencyMatch::SecondService)
            } else if first_dependency == third_service_name
                || second_dependency == third_service_name
            {
                Some(ThreeServiceDependencyMatch::ThirdService)
            } else {
                None
            }
        }
        [first_dependency, second_dependency, third_dependency] => {
            if first_dependency == first_service_name
                || second_dependency == first_service_name
                || third_dependency == first_service_name
            {
                Some(ThreeServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
                || third_dependency == second_service_name
            {
                Some(ThreeServiceDependencyMatch::SecondService)
            } else if first_dependency == third_service_name
                || second_dependency == third_service_name
                || third_dependency == third_service_name
            {
                Some(ThreeServiceDependencyMatch::ThirdService)
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
                Some(ThreeServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
                || third_dependency == second_service_name
                || fourth_dependency == second_service_name
            {
                Some(ThreeServiceDependencyMatch::SecondService)
            } else if first_dependency == third_service_name
                || second_dependency == third_service_name
                || third_dependency == third_service_name
                || fourth_dependency == third_service_name
            {
                Some(ThreeServiceDependencyMatch::ThirdService)
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
                Some(ThreeServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
                || third_dependency == second_service_name
                || fourth_dependency == second_service_name
                || fifth_dependency == second_service_name
            {
                Some(ThreeServiceDependencyMatch::SecondService)
            } else if first_dependency == third_service_name
                || second_dependency == third_service_name
                || third_dependency == third_service_name
                || fourth_dependency == third_service_name
                || fifth_dependency == third_service_name
            {
                Some(ThreeServiceDependencyMatch::ThirdService)
            } else {
                None
            }
        }
        _ => {
            let mut second_service_blocked = false;
            let mut third_service_blocked = false;
            for dependency in dependencies {
                if dependency == first_service_name {
                    return Some(ThreeServiceDependencyMatch::FirstService);
                }
                if dependency == second_service_name {
                    second_service_blocked = true;
                    third_service_blocked = false;
                } else if !second_service_blocked && dependency == third_service_name {
                    third_service_blocked = true;
                }
            }
            if second_service_blocked {
                Some(ThreeServiceDependencyMatch::SecondService)
            } else if third_service_blocked {
                Some(ThreeServiceDependencyMatch::ThirdService)
            } else {
                None
            }
        }
    }
}
