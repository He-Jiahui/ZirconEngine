use super::{assert_contains_all, repo_path, runtime_src_path};

#[path = "provider_boilerplate/feedback.rs"]
mod feedback;
#[path = "provider_boilerplate/full_audit.rs"]
mod full_audit;
#[path = "provider_boilerplate/module_layout.rs"]
mod module_layout;
#[path = "provider_boilerplate/prepare_input.rs"]
mod prepare_input;
#[path = "provider_boilerplate/registration.rs"]
mod registration;
#[path = "provider_boilerplate/update.rs"]
mod update;
fn read_runtime_src(relative: &str) -> String {
    let path = runtime_src_path(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("runtime source should exist at {}: {error}", path.display())
    })
}

fn read_repo(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("repo file should exist at {}: {error}", path.display()))
}

fn assert_not_contains_any(label: &str, source: &str, unexpected: &[&str]) {
    for token in unexpected {
        assert!(
            !source.contains(token),
            "{label} should not contain duplicated provider boilerplate token `{token}`"
        );
    }
}
