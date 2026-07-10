// SPDX-License-Identifier: MPL-2.0

use super::SyscallReturn;
use crate::prelude::*;

/// Reports that restartable sequences are unavailable.
///
/// Returning `ENOSYS` lets libc disable rseq and use its fallback paths.
/// Pretending that registration succeeded would be incorrect because the
/// kernel does not maintain the userspace rseq area during scheduling.
pub fn sys_rseq(
    _rseq_ptr: Vaddr,
    _rseq_len: u32,
    _flags: u32,
    _signature: u32,
    _ctx: &Context,
) -> Result<SyscallReturn> {
    return_errno_with_message!(Errno::ENOSYS, "rseq is not supported");
}
