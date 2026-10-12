use super::{budgets, mounts, sources};

// TODO: [CR-RUNTIME-TESTS-STRUCT-0025] 锁中毒策略父模块与拆分子模块的守卫数量不符；需核对迁移后测试挂载图与预期数量，再调整计数。
#[test]
fn runtime_15_lock_poison_policy_guard_is_folder_backed() {
    let sources = sources::read_lock_poison_sources();

    mounts::assert_parent_mounts_child_owners(&sources);
    mounts::assert_lock_poison_guards_stay_in_children(&sources);
    budgets::assert_lock_poison_owner_budgets(&sources);
}
