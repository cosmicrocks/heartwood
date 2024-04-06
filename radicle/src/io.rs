use std::io;
use libc::{getrlimit, rlimit, setrlimit, RLIMIT_NOFILE};

/// Sets the open file limit to the given value, or the maximum allowed value.
pub fn set_file_limit(n: u64) -> io::Result<u64> {
    let mut rlim = rlimit {
        rlim_cur: 0, // Initial soft limit value
        rlim_max: 0, // Initial hard limit value
    };
    // Get the current limits.
    unsafe {
        if getrlimit(RLIMIT_NOFILE, &mut rlim) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    
    // Convert rlim_cur to u64 for comparison
    let current_limit = rlim.rlim_cur as u64;
    if current_limit >= n {
        return Ok(current_limit);
    }
    
    // Set the soft limit to the given value, up to the hard limit.
    // Ensure that we respect the maximum limit by comparing it as u64.
    let max_limit = rlim.rlim_max as u64;
    rlim.rlim_cur = n.min(max_limit) as libc::rlim_t; // Convert back to libc::rlim_t for assignment
    
    unsafe {
        if setrlimit(RLIMIT_NOFILE, &rlim as *const rlimit) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    
    Ok(rlim.rlim_cur as u64) // Return the new limit as u64
}
