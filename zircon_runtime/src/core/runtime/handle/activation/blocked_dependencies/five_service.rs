use super::super::super::super::descriptors::RegistryName;

pub(in super::super) enum FiveServiceDependencyMatch {
    FirstService,
    SecondService,
    ThirdService,
    FourthService,
    FifthService,
}

pub(in super::super) fn first_blocked_five_service_dependency(
    dependencies: &[RegistryName],
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
    third_service_name: &RegistryName,
    fourth_service_name: &RegistryName,
    fifth_service_name: &RegistryName,
) -> Option<FiveServiceDependencyMatch> {
    match dependencies {
        [] => None,
        [dependency] => {
            if dependency == first_service_name {
                Some(FiveServiceDependencyMatch::FirstService)
            } else if dependency == second_service_name {
                Some(FiveServiceDependencyMatch::SecondService)
            } else if dependency == third_service_name {
                Some(FiveServiceDependencyMatch::ThirdService)
            } else if dependency == fourth_service_name {
                Some(FiveServiceDependencyMatch::FourthService)
            } else if dependency == fifth_service_name {
                Some(FiveServiceDependencyMatch::FifthService)
            } else {
                None
            }
        }
        [first_dependency, second_dependency] => {
            if first_dependency == first_service_name || second_dependency == first_service_name {
                Some(FiveServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
            {
                Some(FiveServiceDependencyMatch::SecondService)
            } else if first_dependency == third_service_name
                || second_dependency == third_service_name
            {
                Some(FiveServiceDependencyMatch::ThirdService)
            } else if first_dependency == fourth_service_name
                || second_dependency == fourth_service_name
            {
                Some(FiveServiceDependencyMatch::FourthService)
            } else if first_dependency == fifth_service_name
                || second_dependency == fifth_service_name
            {
                Some(FiveServiceDependencyMatch::FifthService)
            } else {
                None
            }
        }
        [first_dependency, second_dependency, third_dependency] => {
            if first_dependency == first_service_name
                || second_dependency == first_service_name
                || third_dependency == first_service_name
            {
                Some(FiveServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
                || third_dependency == second_service_name
            {
                Some(FiveServiceDependencyMatch::SecondService)
            } else if first_dependency == third_service_name
                || second_dependency == third_service_name
                || third_dependency == third_service_name
            {
                Some(FiveServiceDependencyMatch::ThirdService)
            } else if first_dependency == fourth_service_name
                || second_dependency == fourth_service_name
                || third_dependency == fourth_service_name
            {
                Some(FiveServiceDependencyMatch::FourthService)
            } else if first_dependency == fifth_service_name
                || second_dependency == fifth_service_name
                || third_dependency == fifth_service_name
            {
                Some(FiveServiceDependencyMatch::FifthService)
            } else {
                None
            }
        }
        [first_dependency, second_dependency, third_dependency, fourth_dependency] => {
            if first_dependency == first_service_name
                || second_dependency == first_service_name
                || third_dependency == first_service_name
                || fourth_dependency == first_service_name
            {
                Some(FiveServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
                || third_dependency == second_service_name
                || fourth_dependency == second_service_name
            {
                Some(FiveServiceDependencyMatch::SecondService)
            } else if first_dependency == third_service_name
                || second_dependency == third_service_name
                || third_dependency == third_service_name
                || fourth_dependency == third_service_name
            {
                Some(FiveServiceDependencyMatch::ThirdService)
            } else if first_dependency == fourth_service_name
                || second_dependency == fourth_service_name
                || third_dependency == fourth_service_name
                || fourth_dependency == fourth_service_name
            {
                Some(FiveServiceDependencyMatch::FourthService)
            } else if first_dependency == fifth_service_name
                || second_dependency == fifth_service_name
                || third_dependency == fifth_service_name
                || fourth_dependency == fifth_service_name
            {
                Some(FiveServiceDependencyMatch::FifthService)
            } else {
                None
            }
        }
        [first_dependency, second_dependency, third_dependency, fourth_dependency, fifth_dependency] => {
            if first_dependency == first_service_name
                || second_dependency == first_service_name
                || third_dependency == first_service_name
                || fourth_dependency == first_service_name
                || fifth_dependency == first_service_name
            {
                Some(FiveServiceDependencyMatch::FirstService)
            } else if first_dependency == second_service_name
                || second_dependency == second_service_name
                || third_dependency == second_service_name
                || fourth_dependency == second_service_name
                || fifth_dependency == second_service_name
            {
                Some(FiveServiceDependencyMatch::SecondService)
            } else if first_dependency == third_service_name
                || second_dependency == third_service_name
                || third_dependency == third_service_name
                || fourth_dependency == third_service_name
                || fifth_dependency == third_service_name
            {
                Some(FiveServiceDependencyMatch::ThirdService)
            } else if first_dependency == fourth_service_name
                || second_dependency == fourth_service_name
                || third_dependency == fourth_service_name
                || fourth_dependency == fourth_service_name
                || fifth_dependency == fourth_service_name
            {
                Some(FiveServiceDependencyMatch::FourthService)
            } else if first_dependency == fifth_service_name
                || second_dependency == fifth_service_name
                || third_dependency == fifth_service_name
                || fourth_dependency == fifth_service_name
                || fifth_dependency == fifth_service_name
            {
                Some(FiveServiceDependencyMatch::FifthService)
            } else {
                None
            }
        }
        _ => {
            let mut second_service_blocked = false;
            let mut third_service_blocked = false;
            let mut fourth_service_blocked = false;
            let mut fifth_service_blocked = false;
            for dependency in dependencies {
                if dependency == first_service_name {
                    return Some(FiveServiceDependencyMatch::FirstService);
                }
                if dependency == second_service_name {
                    second_service_blocked = true;
                    third_service_blocked = false;
                    fourth_service_blocked = false;
                    fifth_service_blocked = false;
                } else if !second_service_blocked && dependency == third_service_name {
                    third_service_blocked = true;
                    fourth_service_blocked = false;
                    fifth_service_blocked = false;
                } else if !second_service_blocked
                    && !third_service_blocked
                    && dependency == fourth_service_name
                {
                    fourth_service_blocked = true;
                    fifth_service_blocked = false;
                } else if !second_service_blocked
                    && !third_service_blocked
                    && !fourth_service_blocked
                    && dependency == fifth_service_name
                {
                    fifth_service_blocked = true;
                }
            }
            if second_service_blocked {
                Some(FiveServiceDependencyMatch::SecondService)
            } else if third_service_blocked {
                Some(FiveServiceDependencyMatch::ThirdService)
            } else if fourth_service_blocked {
                Some(FiveServiceDependencyMatch::FourthService)
            } else if fifth_service_blocked {
                Some(FiveServiceDependencyMatch::FifthService)
            } else {
                None
            }
        }
    }
}
