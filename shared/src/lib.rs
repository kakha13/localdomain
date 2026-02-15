pub mod domain;
pub mod protocol;

/// Create a `Command` that won't spawn a visible console window on Windows.
/// On non-Windows platforms this is identical to `std::process::Command::new()`.
#[cfg(windows)]
pub fn silent_cmd<S: AsRef<std::ffi::OsStr>>(program: S) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut cmd = std::process::Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(windows))]
pub fn silent_cmd<S: AsRef<std::ffi::OsStr>>(program: S) -> std::process::Command {
    std::process::Command::new(program)
}
