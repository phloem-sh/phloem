use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::process::Command;
use which::which;

pub struct EnvironmentDetector;

impl Default for EnvironmentDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_environment(&self) -> Result<HashMap<String, String>> {
        let mut env_info = HashMap::new();

        // Basic system information
        env_info.insert("os".to_string(), env::consts::OS.to_string());
        env_info.insert("arch".to_string(), env::consts::ARCH.to_string());

        // Shell information
        if let Ok(shell) = env::var("SHELL") {
            env_info.insert("shell".to_string(), shell);
        }

        // Terminal information
        if let Ok(term) = env::var("TERM") {
            env_info.insert("terminal".to_string(), term);
        }

        // Current working directory
        if let Ok(pwd) = env::current_dir() {
            env_info.insert("pwd".to_string(), pwd.display().to_string());
        }

        // Detect available tools
        let available_tools = self.detect_available_tools();
        env_info.insert("available_tools".to_string(), available_tools.join(","));

        // Container runtime detection
        if let Some(container_runtime) = self.detect_container_runtime() {
            env_info.insert("container_runtime".to_string(), container_runtime);
        }

        // Cloud provider detection
        if let Some(cloud_provider) = self.detect_cloud_provider() {
            env_info.insert("cloud_provider".to_string(), cloud_provider);
        }

        // Kubernetes context
        if let Some(k8s_context) = self.detect_kubernetes_context() {
            env_info.insert("kubernetes_context".to_string(), k8s_context);
        }

        Ok(env_info)
    }

    fn detect_available_tools(&self) -> Vec<String> {
        let mut available = Vec::new();

        // Check common development and system tools
        let common_tools = [
            // Basic system tools
            "ls",
            "cat",
            "grep",
            "find",
            "sort",
            "head",
            "tail",
            "curl",
            "wget",
            "ps",
            "top",
            "df",
            "du",
            // Development tools
            "git",
            "docker",
            "kubectl",
            "npm",
            "yarn",
            "python",
            "python3",
            "pip",
            "pip3",
            "cargo",
            "rustc",
            "go",
            "java",
            "mvn",
            "gradle",
            "make",
            "cmake",
            // Database tools
            "mysql",
            "psql",
            "sqlite3",
            "mongo",
            "redis-cli",
            // Graph database tools
            "cypher-shell",
            "mgconsole",
            "neo4j-shell",
            "neo4j",
            // Editors and utilities
            "vim",
            "nano",
            "emacs",
            "code",
            "ssh",
            "scp",
            "rsync",
            "tar",
            "zip",
            "unzip",
            "awk",
            "sed",
            "xargs",
            "jq",
            "yq",
            "htop",
            "lsof",
            "which",
            "whereis",
        ];

        for tool in &common_tools {
            if which(tool).is_ok() {
                available.push(tool.to_string());
            }
        }

        // Also scan common binary directories for additional tools
        let bin_dirs = ["/usr/local/bin", "/usr/bin", "/bin"];
        for dir in &bin_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            if let Some(name) = entry.file_name().to_str() {
                                if !available.contains(&name.to_string())
                                    && !name.starts_with('.')
                                    && name.len() > 1
                                    && name.len() < 20
                                {
                                    // Reasonable executable name length
                                    available.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Limit to prevent overwhelming the prompt
        available.sort();
        available.dedup();
        available.truncate(100);

        available
    }

    fn detect_container_runtime(&self) -> Option<String> {
        if which("docker").is_ok() {
            // Check if Docker is running
            if let Ok(output) = Command::new("docker").arg("info").output() {
                if output.status.success() {
                    return Some("Docker".to_string());
                }
            }
        }

        if which("podman").is_ok() {
            return Some("Podman".to_string());
        }

        None
    }

    fn detect_cloud_provider(&self) -> Option<String> {
        // AWS detection
        if env::var("AWS_PROFILE").is_ok() || env::var("AWS_DEFAULT_REGION").is_ok() {
            return Some("AWS".to_string());
        }

        if let Ok(home) = env::var("HOME") {
            let aws_config = std::path::Path::new(&home).join(".aws");
            if aws_config.exists() {
                return Some("AWS".to_string());
            }
        }

        // GCP detection
        if env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok() {
            return Some("GCP".to_string());
        }

        if let Ok(home) = env::var("HOME") {
            let gcp_config = std::path::Path::new(&home).join(".config").join("gcloud");
            if gcp_config.exists() {
                return Some("GCP".to_string());
            }
        }

        // Azure detection
        if let Ok(home) = env::var("HOME") {
            let azure_config = std::path::Path::new(&home).join(".azure");
            if azure_config.exists() {
                return Some("Azure".to_string());
            }
        }

        None
    }

    fn detect_kubernetes_context(&self) -> Option<String> {
        if which("kubectl").is_ok() {
            if let Ok(output) = Command::new("kubectl")
                .args(["config", "current-context"])
                .output()
            {
                if output.status.success() {
                    let context = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !context.is_empty() {
                        return Some(context);
                    }
                }
            }
        }

        None
    }
}
