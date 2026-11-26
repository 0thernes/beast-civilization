//! Genuine Win32 OS Kernel Job Object Sandbox (`crates/beast-root/src/win32_job.rs`).
//! Zero external crates: links directly to `kernel32.dll` (`CreateJobObjectW`, `SetInformationJobObject`,
//! `AssignProcessToJobObject`, `IsProcessInJob`, `CloseHandle`) to enforce OS-kernel hard memory caps
//! (`128 MiB`) and `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` on spawned Agent-Zone child processes!

use crate::Ring0Root;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Win32JobReport {
    pub supported_on_os: bool,
    pub job_created: bool,
    pub child_pid: u32,
    pub assigned_to_job: bool,
    pub verified_in_job: bool,
    pub memory_cap_bytes: usize,
    pub secret_scrubbed: bool,
}

#[cfg(windows)]
mod win32_ffi {
    use std::ffi::c_void;

    pub type HANDLE = *mut c_void;
    pub type BOOL = i32;
    pub const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: u32 = 9;
    pub const JOB_OBJECT_LIMIT_PROCESS_MEMORY: u32 = 0x0000_0100;
    pub const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;

    #[repr(C)]
    #[derive(Default)]
    pub struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
        pub per_process_user_time_limit: i64,
        pub per_job_user_time_limit: i64,
        pub limit_flags: u32,
        pub minimum_working_set_size: usize,
        pub maximum_working_set_size: usize,
        pub active_process_limit: u32,
        pub affinity: usize,
        pub priority_class: u32,
        pub scheduling_class: u32,
    }

    #[repr(C)]
    #[derive(Default)]
    pub struct IO_COUNTERS {
        pub read_operation_count: u64,
        pub write_operation_count: u64,
        pub other_operation_count: u64,
        pub read_transfer_count: u64,
        pub write_transfer_count: u64,
        pub other_transfer_count: u64,
    }

    #[repr(C)]
    #[derive(Default)]
    pub struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        pub basic_limit_information: JOBOBJECT_BASIC_LIMIT_INFORMATION,
        pub io_info: IO_COUNTERS,
        pub process_memory_limit: usize,
        pub job_memory_limit: usize,
        pub peak_process_memory_used: usize,
        pub peak_job_memory_used: usize,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn CreateJobObjectW(lp_job_attributes: *mut c_void, lp_name: *const u16) -> HANDLE;
        pub fn SetInformationJobObject(
            h_job: HANDLE,
            job_object_information_class: u32,
            lp_job_object_information: *const c_void,
            cb_job_object_information_length: u32,
        ) -> BOOL;
        pub fn AssignProcessToJobObject(h_job: HANDLE, h_process: HANDLE) -> BOOL;
        pub fn IsProcessInJob(
            process_handle: HANDLE,
            job_handle: HANDLE,
            result: *mut BOOL,
        ) -> BOOL;
        pub fn CloseHandle(h_object: HANDLE) -> BOOL;
    }
}

#[cfg(windows)]
pub fn verify_win32_kernel_job_sandbox(
    root: &Ring0Root,
    beast_exe: &Path,
    memory_cap_bytes: usize,
) -> Result<Win32JobReport, String> {
    use std::os::windows::io::AsRawHandle;
    use win32_ffi::*;

    let h_job = unsafe { CreateJobObjectW(std::ptr::null_mut(), std::ptr::null()) };
    if h_job.is_null() {
        return Err("CreateJobObjectW failed".into());
    }

    let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    info.basic_limit_information.limit_flags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
    info.process_memory_limit = memory_cap_bytes;

    let set_ok = unsafe {
        SetInformationJobObject(
            h_job,
            JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
            (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };
    if set_ok == 0 {
        unsafe { CloseHandle(h_job) };
        return Err("SetInformationJobObject failed".into());
    }

    let prog = beast_exe.to_str().ok_or_else(|| {
        unsafe { CloseHandle(h_job) };
        "non-UTF8 exe path".to_string()
    })?;
    let mut cmd = root.build_sandboxed_command(prog, &["worker-handshake", "win32-job-v0.5"]);
    cmd.env("BEAST_ZONE", "AGENT_WIN32_JOB");
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let child = cmd.spawn().map_err(|e| {
        unsafe { CloseHandle(h_job) };
        format!("spawn child for job object: {e}")
    })?;
    let child_pid = child.id();
    let child_handle = child.as_raw_handle() as HANDLE;

    let assigned = unsafe { AssignProcessToJobObject(h_job, child_handle) } != 0;
    let mut in_job_flag: BOOL = 0;
    let query_ok = unsafe { IsProcessInJob(child_handle, h_job, &mut in_job_flag) } != 0;
    let verified_in_job = query_ok && in_job_flag != 0;

    let out = child.wait_with_output().map_err(|e| {
        unsafe { CloseHandle(h_job) };
        format!("wait job child: {e}")
    })?;
    unsafe { CloseHandle(h_job) };

    if !out.status.success() {
        return Err(format!("job child exited with {}", out.status));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let secret_scrubbed = stdout.contains("WORKER_OK") && stdout.contains("secret_leaked=false");
    if !assigned || !verified_in_job || !secret_scrubbed {
        return Err(format!(
            "Win32 Job Object verification failed: assigned={assigned} in_job={verified_in_job} scrubbed={secret_scrubbed} out={stdout:?}"
        ));
    }

    Ok(Win32JobReport {
        supported_on_os: true,
        job_created: true,
        child_pid,
        assigned_to_job: assigned,
        verified_in_job,
        memory_cap_bytes,
        secret_scrubbed,
    })
}

#[cfg(not(windows))]
pub fn verify_win32_kernel_job_sandbox(
    _root: &Ring0Root,
    _beast_exe: &Path,
    memory_cap_bytes: usize,
) -> Result<Win32JobReport, String> {
    Ok(Win32JobReport {
        supported_on_os: false,
        job_created: false,
        child_pid: 0,
        assigned_to_job: false,
        verified_in_job: false,
        memory_cap_bytes,
        secret_scrubbed: true,
    })
}
