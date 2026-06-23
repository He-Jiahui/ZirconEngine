mod backend_owners;
mod gpu_resource_owners;
mod module_layout;
mod renderer_output_accessors;
mod resource_streamer_cleanup;

use super::{repo_path, runtime_src_path};

fn read_runtime_src(relative: &str) -> String {
    let path = runtime_src_path(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("runtime source should exist at {}: {error}", path.display())
    })
}

fn read_repo(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("repo source should exist at {}: {error}", path.display()))
}
