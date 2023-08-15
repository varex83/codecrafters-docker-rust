use std::env;
use std::ffi::CStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::{exit, Stdio};
use anyhow::{Context, Result};
use tokio::process::Command;
use libc::chroot;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let dir_name = "rootfs";

    let cwd = Path::new(dir_name);

    println!("Current working directory: {:?}", cwd);

    if let Err(e) = std::fs::create_dir(cwd.clone()) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(e.into());
        }
    } else {
        println!("Created directory '{}'", dir_name);
    }

    unsafe {
        chroot(CStr::from_bytes_with_nul_unchecked(
            cwd.clone().as_os_str().as_bytes().to_vec().as_slice(),
        ).as_ptr())
    };

    println!("Changed root to '{}'", cwd.as_os_str().to_str().unwrap());

    env::set_current_dir(cwd.clone()).with_context(|| {
        format!("Tried to change directory to '/'")
    })?;

    let mut child = Command::new(command)
        .args(command_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    let exit_status = child.wait().await?;

    // Exit with the same exit code as the child process.
    exit(exit_status.code().unwrap_or(1));
}