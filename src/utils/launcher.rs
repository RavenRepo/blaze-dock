//! Application launcher utility
//!
//! Provides async, non-blocking application launching to ensure
//! the dock UI never freezes when starting applications.

use anyhow::{Context, Result};
use log::{debug, info};
use std::process::Stdio;
use tokio::process::Command;

/// Launch an application command asynchronously
///
/// This function spawns the command in a detached process so:
/// 1. The dock doesn't wait for the application to exit
/// 2. The UI remains responsive during launch
/// 3. The child process isn't killed when the dock closes
///
/// # Arguments
/// * `command` - The command to execute (can include arguments)
///
/// # Returns
/// * `Ok(())` if the command was successfully spawned
/// * `Err` if the command failed to start
pub async fn launch_command(command: &str) -> Result<()> {
    debug!("Launching command: {}", command);

    // Parse the command into program and arguments
    let parts: Vec<&str> = command.split_whitespace().collect();
    
    if parts.is_empty() {
        anyhow::bail!("Empty command provided");
    }

    let program = parts[0];
    let args = &parts[1..];

    // Spawn the process detached from the dock
    let result = Command::new(program)
        .args(args)
        // Don't inherit stdin/stdout/stderr - fully detach
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        // Create a new process group so killing the dock doesn't kill apps
        .process_group(0)
        .spawn()
        .context(format!("Failed to spawn command: {}", command))?;

    info!(
        "Successfully launched '{}' (PID: {:?})",
        program,
        result.id()
    );

    Ok(())
}

/// Launch an application from its .desktop file
///
/// This provides richer integration by parsing the Exec field
/// and handling special desktop entry syntax.
///
/// # Arguments
/// * `desktop_file_path` - Path to the .desktop file
pub async fn launch_desktop_file(desktop_file_path: &str) -> Result<()> {
    use crate::utils::desktop_entry::DesktopEntry;

    debug!("Launching from desktop file: {}", desktop_file_path);

    let entry = DesktopEntry::parse(desktop_file_path)
        .context("Failed to parse desktop file")?;

    // Get the exec command, stripping field codes like %u, %F, etc.
    let exec = entry.exec_command()
        .context("Desktop file has no Exec field")?;

    launch_command(&exec).await
}

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_parsing() {
        // Basic parsing test (doesn't actually launch)
        let parts: Vec<&str> = "firefox --new-window".split_whitespace().collect();
        assert_eq!(parts[0], "firefox");
        assert_eq!(parts[1], "--new-window");
    }
}

