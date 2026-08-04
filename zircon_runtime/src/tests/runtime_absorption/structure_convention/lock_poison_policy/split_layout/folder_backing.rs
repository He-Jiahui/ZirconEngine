use super::{budgets, mounts, sources};

#[test]
fn runtime_15_lock_poison_policy_guard_is_folder_backed() {
    let sources = sources::read_lock_poison_sources();

    mounts::assert_parent_mounts_child_owners(&sources);
    mounts::assert_lock_poison_guards_stay_in_children(&sources);
    budgets::assert_lock_poison_owner_budgets(&sources);
}
