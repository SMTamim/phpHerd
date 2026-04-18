use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Failed,
}

struct ManagedProcess {
    name: String,
    child: Child,
    command: String,
    args: Vec<String>,
    working_dir: Option<PathBuf>,
}

pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<String, ManagedProcess>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(
        &self,
        name: &str,
        command: &str,
        args: &[&str],
        working_dir: Option<PathBuf>,
        env: Option<HashMap<String, String>>,
    ) -> Result<u32> {
        // Stop existing process with same name
        self.stop(name).await.ok();

        let mut cmd = Command::new(command);
        cmd.args(args);

        if let Some(dir) = &working_dir {
            cmd.current_dir(dir);
        }

        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let child = cmd.spawn()?;
        let pid = child.id().unwrap_or(0);

        let managed = ManagedProcess {
            name: name.to_string(),
            child,
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            working_dir,
        };

        self.processes
            .write()
            .await
            .insert(name.to_string(), managed);

        tracing::info!("Started process '{}' with PID {}", name, pid);
        Ok(pid)
    }

    pub async fn stop(&self, name: &str) -> Result<()> {
        let mut processes = self.processes.write().await;
        if let Some(mut process) = processes.remove(name) {
            tracing::info!("Stopping process '{}'", name);
            process.child.kill().await?;
            tracing::info!("Process '{}' stopped", name);
        }
        Ok(())
    }

    pub async fn restart(&self, name: &str) -> Result<u32> {
        let (command, args, working_dir) = {
            let processes = self.processes.read().await;
            if let Some(process) = processes.get(name) {
                (
                    process.command.clone(),
                    process.args.clone(),
                    process.working_dir.clone(),
                )
            } else {
                return Err(anyhow::anyhow!("Process '{}' not found", name));
            }
        };

        self.stop(name).await?;
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        self.start(name, &command, &args_refs, working_dir, None)
            .await
    }

    pub async fn status(&self, name: &str) -> ProcessStatus {
        let processes = self.processes.read().await;
        if processes.contains_key(name) {
            ProcessStatus::Running
        } else {
            ProcessStatus::Stopped
        }
    }

    pub async fn list(&self) -> Vec<ProcessInfo> {
        let processes = self.processes.read().await;
        processes
            .values()
            .map(|p| ProcessInfo {
                name: p.name.clone(),
                pid: p.child.id().unwrap_or(0),
                status: ProcessStatus::Running,
            })
            .collect()
    }

    pub async fn stop_all(&self) -> Result<()> {
        let names: Vec<String> = {
            let processes = self.processes.read().await;
            processes.keys().cloned().collect()
        };

        for name in names {
            self.stop(&name).await?;
        }
        Ok(())
    }
}
