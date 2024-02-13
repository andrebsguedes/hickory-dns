use std::{
    process::{Command, Stdio},
    sync::atomic::{self, AtomicUsize},
};

use crate::Result;

const NETWORK_NAME: &str = "dnssec-network";

/// Represents a network in which to put containers into.
pub struct Network {
    name: String,
}

impl Network {
    pub fn new() -> Result<Self> {
        let id = network_count();
        let network_name = format!("{NETWORK_NAME}-{id}");

        let mut command = Command::new("docker");
        command
            .args(["network", "create"])
            .args(["--internal"])
            .arg(&network_name);

        // create network
        let output = command.output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "--- STDOUT ---\n{stdout}\n--- STDERR ---\n{stderr}"
        );

        // inspect & parse network details

        Ok(Self { name: network_name })
    }

    /// Returns the name of the network.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// Collects all important configs.
pub struct NetworkConfig {
    /// The CIDR subnet mask, e.g. "172.21.0.0/16"
    subnet: String,
}

///
fn get_network_config(network_name: &str) -> Result<NetworkConfig> {
    let mut command = Command::new("docker");
    command
        .args([
            "network",
            "inspect",
            "-f",
            "{{range .IPAM.Config}}{{.Subnet}}{{end}}",
        ])
        .arg(network_name);

    let output = command.output()?;
    if !output.status.success() {
        return Err(format!("{command:?} failed").into());
    }

    let subnet = std::str::from_utf8(&output.stdout)?.trim().to_string();
    Ok(NetworkConfig { subnet })
}

/// This ensure the Docket network is deleted after the test runner process ends.
impl Drop for Network {
    fn drop(&mut self) {
        // Remove the network
        // TODO check if all containers need to disconnect first
        let _ = Command::new("docker")
            .args(["network", "rm", "--force", self.name.as_str()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn network_count() -> usize {
    static COUNT: AtomicUsize = AtomicUsize::new(1);

    COUNT.fetch_add(1, atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_works() -> Result<()> {
        assert!(Network::new().is_ok());
        Ok(())
    }

    #[test]
    fn network_subnet_works() -> Result<()> {
        let network = Network::new().expect("Failed to create network");
        let config = get_network_config(network.name());
        assert!(config.is_ok());
        Ok(())
    }
}
