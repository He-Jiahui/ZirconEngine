//! Validation and release helpers for runtime-owned ABI results.
//!
//! 本模块验证 `ZrOwnedResultV2` 的主机可表示形态，并通过产生分配的 session release callback
//! 释放非空输出。App 和 Editor 的消费者负责保证 provider/session 在读取、释放期间仍然存活。

use zircon_runtime_interface::{
    ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeReleaseAllocationFnV2, ZrRuntimeSessionHandle,
    ZrStatus,
};

use super::RuntimeForeignOutputError;

#[derive(Clone, Copy)]
/// 将一个 Runtime session 与释放其输出分配的 ABI 回调配对。
///
/// 此值可复制，因此本身不拥有 session，也不会阻止 provider unload；调用者须维持下方
/// `new` 的存活前提。
pub struct RuntimeOwnedOutputReleaser {
    /// 运行时分配所属的 session。
    session: ZrRuntimeSessionHandle,
    /// 与该 session 同属一个 provider 的释放入口。
    release: ZrRuntimeReleaseAllocationFnV2,
}

impl RuntimeOwnedOutputReleaser {
    /// Binds a session handle to its runtime allocation-release function.
    ///
    /// ```compile_fail
    /// use zircon_runtime_host::foreign_output::RuntimeOwnedOutputReleaser;
    /// use zircon_runtime_interface::{ZrRuntimeReleaseAllocationFnV2, ZrRuntimeSessionHandle};
    ///
    /// fn construct(
    ///     session: ZrRuntimeSessionHandle,
    ///     release: ZrRuntimeReleaseAllocationFnV2,
    /// ) {
    ///     let _ = RuntimeOwnedOutputReleaser::new(session, release);
    /// }
    /// ```
    ///
    /// 此构造器只保存 handle 和函数指针，不取得 session 租约；每次使用时仍须满足上述同一
    /// provider、session 与回调持续存活的前提。
    ///
    /// # Safety
    ///
    /// `release` must belong to the provider that owns `session`, and that provider and session
    /// must remain live whenever this releaser is used.
    pub const unsafe fn new(
        session: ZrRuntimeSessionHandle,
        release: ZrRuntimeReleaseAllocationFnV2,
    ) -> Self {
        Self { session, release }
    }

    /// 返回此释放器绑定的 session handle。
    pub const fn session(self) -> ZrRuntimeSessionHandle {
        self.session
    }

    /// 将 allocation ID 和绑定的 session 交给 provider 的 ABI 回调。
    fn release(self, allocation: ZrRuntimeAllocationId) -> ZrStatus {
        // SAFETY: releaser 的构造契约保证回调属于该 session 的 provider；调用方须令二者在调用期间存活，
        // 并传入该 provider 返回且仍由调用方独占释放权的 allocation ID。
        unsafe { (self.release)(self.session, allocation) }
    }
}

/// Releases a runtime-owned result through its originating session.
///
/// 调用者须确保输出来自该释放器所绑定且仍存活的 provider/session，并独占该 allocation 的一次
/// 释放权。空输出直接成功；非空输出须带有效 allocation ID，释放失败转换为 Runtime 调用错误。
///
/// # Safety
///
/// `output` must have been returned by the same live provider and session bound to `releaser`.
/// The caller must hold the unique authority to release its allocation exactly once. Any failure
/// diagnostics returned by the release callback must remain synchronously readable.
pub unsafe fn release_owned_result(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    operation: &'static str,
) -> Result<(), RuntimeForeignOutputError> {
    if output.is_empty() {
        return Ok(());
    }
    if !output.allocation.is_valid() {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned runtime-owned storage without an allocation ID"
        )));
    }
    // SAFETY: 此 unsafe API 的调用者保证 releaser 所绑定的 provider/session 仍存活且拥有该 allocation；
    // release 回调产生的失败诊断按 Safety 契约在下面的同步 from_status 读取期间保持有效。
    match unsafe {
        RuntimeForeignOutputError::from_status(releaser.release(output.allocation), operation)
    } {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/// 该辅助函数先释放输出；释放成功后返回原错误，释放失败时把清理诊断附加到原错误。
///
/// # Safety
///
/// The safety requirements of [`release_owned_result`] apply.
pub unsafe fn release_owned_result_after_error<T>(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    error: RuntimeForeignOutputError,
    release_operation: &'static str,
) -> Result<T, RuntimeForeignOutputError> {
    // SAFETY: 该函数复用其 Safety 契约要求的同一 provider/session 和唯一释放权，
    // 并将这些前提原样传递给 release_owned_result。
    match unsafe { release_owned_result(output, releaser, release_operation) } {
        Ok(()) => Err(error),
        Err(release_error) => Err(error.with_cleanup_failure(&release_error)),
    }
}

/// 无论 `result` 成功与否都会先释放输出；仅在结果和释放均成功时保留 `T`，并按原始错误优先
/// 的顺序合并同时发生的业务错误与清理错误。
///
/// # Safety
///
/// The safety requirements of [`release_owned_result`] apply.
pub unsafe fn release_owned_result_after_result<T>(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    result: Result<T, RuntimeForeignOutputError>,
    release_operation: &'static str,
) -> Result<T, RuntimeForeignOutputError> {
    // SAFETY: 本函数沿用 release_owned_result 的同一存活来源及唯一释放权前提。
    match (result, unsafe {
        release_owned_result(output, releaser, release_operation)
    }) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(release_error)) => Err(release_error),
        (Err(error), Err(release_error)) => Err(error.with_cleanup_failure(&release_error)),
    }
}

/// 校验结果长度、指针与 allocation ID 的组合是否合法。
///
/// 此函数只检查元数据，不解引用 `data`。非空有效结果返回可用于切片构造的主机长度；规范空
/// 结果要求空指针、零长度和无效 ID，返回 `Ok(0)`。
pub fn validate_owned_result(
    output: &ZrOwnedResultV2,
    operation: &'static str,
) -> Result<usize, RuntimeForeignOutputError> {
    let len = usize::try_from(output.len).map_err(|_| {
        RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned a length that exceeds the host address space"
        ))
    })?;
    if len > isize::MAX as usize {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned length {len} above the maximum Rust slice length"
        )));
    }
    if output.data.is_null() {
        return if len == 0 && !output.allocation.is_valid() {
            Ok(0)
        } else {
            Err(RuntimeForeignOutputError::protocol_violation(format!(
                "{operation} returned null data with len {} and allocation {}",
                output.len,
                output.allocation.raw()
            )))
        };
    }
    if len == 0 {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned non-null data for an empty payload"
        )));
    }
    if !output.allocation.is_valid() {
        return Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned owned storage without an allocation ID"
        )));
    }
    Ok(len)
}

/// 成功时把仍待调用者解码和释放的输出原样返回；失败时依据该输出的 allocation ID 尝试清理，
/// 再传播验证错误或合并清理错误。
///
/// # Safety
///
/// If validation fails, the safety requirements of [`release_owned_result`] apply.
pub unsafe fn validate_owned_result_releasing_on_error(
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
    operation: &'static str,
    release_operation: &'static str,
) -> Result<ZrOwnedResultV2, RuntimeForeignOutputError> {
    match validate_owned_result(&output, operation) {
        Ok(_) => Ok(output),
        // SAFETY: 仅验证失败分支需要清理；该 unsafe API 的调用者保证 output/releaser 来源相同且存活，
        // 并将其唯一释放权交给此路径。
        Err(error) => unsafe {
            release_owned_result_after_error(output, releaser, error, release_operation)
        },
    }
}
