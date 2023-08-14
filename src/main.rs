use std::process::{exit, Stdio};
use anyhow::{Context, Result};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

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
