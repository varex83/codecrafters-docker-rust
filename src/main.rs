use std::env;
use std::ffi::CStr;
use tokio::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::{exit, Stdio};
use anyhow::{Context, Result};
use tokio::process::Command;
use libc::{chroot, rand};
use tokio::fs::create_dir_all;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let dir_name = gen_chroot_dirname();

    generate_chroot_dir(&dir_name).await?;

    copy_binaries_to_chroot(command.clone(), &dir_name).await?;

    change_root(&dir_name)?;

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

async fn generate_chroot_dir(dir_name: &str) -> Result<()> {
    let cwd = env::current_dir()?
        .join(dir_name);

    create_dir_all(cwd.clone()).await?;

    let dev = cwd.clone().join("dev");

    create_dir_all(dev.clone()).await?;

    let dev_null = cwd.clone().join("dev").join("null");

    File::create(dev_null.clone()).await?;

    let bin = cwd.clone().join("usr/local/bin");

    create_dir_all(bin.clone()).await?;

    Ok(())
}

fn change_root(dir_name: &str) -> Result<()> {
    let cwd = env::current_dir()?
        .join(dir_name);

    unsafe {
        chroot(CStr::from_bytes_with_nul_unchecked(
            cwd.clone().as_os_str().as_bytes().to_vec().as_slice(),
        ).as_ptr())
    };

    env::set_current_dir("/").with_context(|| {
        format!("Tried to change directory to '/'")
    })?;

    Ok(())
}

async fn copy_binaries_to_chroot(cmd: String, dir_name: &str) -> Result<()> {
    let bin_path = Path::new(cmd.as_str());
    let dest_path = Path::new(dir_name)
        .join(cmd.trim_start_matches("/"));

    tokio::fs::copy(bin_path, &dest_path).await
        .with_context(|| format!("Failed to copy from '{}' to '{}'", bin_path.display(), dest_path.display()))?;

    Ok(())
}


fn gen_chroot_dirname() -> String {
    let suffix = unsafe {
        rand()
    }.to_string();

    format!("chroot-{}", suffix)
}