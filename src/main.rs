use std::env;
use std::ffi::CStr;
use std::os::unix::ffi::OsStrExt;
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

    let cwd = env::current_dir()?
        .join(dir_name);

    if let Err(e) = std::fs::create_dir(cwd.clone()) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(e.into());
        }
    }

    let dev_null = cwd.clone().join("dev/null");

    if let Err(e) = std::fs::create_dir_all(dev_null.clone().parent().unwrap()) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(e.into());
        }
    }

    unsafe {
        chroot(CStr::from_bytes_with_nul_unchecked(
            cwd.clone().as_os_str().as_bytes().to_vec().as_slice(),
        ).as_ptr())
    };

    env::set_current_dir("/").with_context(|| {
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

    exit(exit_status.code().unwrap_or(1));
}