use anyhow::{Context, Result};
use std::process::Command;

/// A spawned mascot with its runtime ID and prototype name.
#[derive(Debug, Clone)]
pub struct ActiveMascot {
    pub id: u32,
    pub name: String,
}

/// Returns every imported prototype name.
/// Runs: shimejictl prototypes list
pub fn list_prototypes() -> Result<Vec<String>> {
    let out = Command::new("shimejictl")
        .args(["prototypes", "list"])
        .output()
        .context("Failed to run shimejictl — is wl_shimeji installed and running?")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("shimejictl prototypes list failed: {}", stderr);
    }

    let names = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Ok(names)
}

/// Returns all currently active (spawned) mascots.
/// Runs: shimejictl list
///
/// Expected output format (one mascot per line):
///   #1 Neuroling
///   #2 Eviling
pub fn list_active() -> Result<Vec<ActiveMascot>> {
    let out = Command::new("shimejictl")
        .arg("list")
        .output()
        .context("Failed to run shimejictl")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("shimejictl list failed: {}", stderr);
    }

    let mascots = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            // Parse "#<id> <name>"
            let mut parts = line.splitn(2, ' ');
            let id_part = parts.next()?;
            let name = parts.next()?.trim().to_string();
            let id: u32 = id_part.trim_start_matches('#').parse().ok()?;
            Some(ActiveMascot { id, name })
        })
        .collect();

    Ok(mascots)
}

/// Spawns one mascot by prototype name.
/// Runs: shimejictl summon <name>
pub fn spawn_mascot(name: &str) -> Result<()> {
    let status = Command::new("shimejictl")
        .args(["summon", name])
        .status()
        .context("Failed to run shimejictl")?;

    if !status.success() {
        anyhow::bail!("Failed to summon mascot '{}'", name);
    }

    Ok(())
}

/// Dismisses one active mascot by its ID.
/// Runs: shimejictl mascot dismiss -i <id>
pub fn dismiss_mascot(id: u32) -> Result<()> {
    let status = Command::new("shimejictl")
        .args(["mascot", "dismiss", "-i", &id.to_string()])
        .status()
        .context("Failed to run shimejictl")?;

    if !status.success() {
        anyhow::bail!("Failed to dismiss mascot #{}", id);
    }

    Ok(())
}

/// Dismisses all active mascots.
/// Runs: shimejictl dismiss -a
pub fn dismiss_all() -> Result<()> {
    let status = Command::new("shimejictl")
        .args(["dismiss", "-a"])
        .status()
        .context("Failed to run shimejictl")?;

    if !status.success() {
        anyhow::bail!("Failed to dismiss all mascots");
    }

    Ok(())
}
