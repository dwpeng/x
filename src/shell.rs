use anyhow::{Result, anyhow};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Unknown,
}

/// Detect the current shell type
pub fn detect_shell() -> ShellType {
    // Try to get shell from SHELL environment variable
    if let Ok(shell_path) = env::var("SHELL") {
        let shell_name = Path::new(&shell_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match shell_name {
            "bash" => return ShellType::Bash,
            "zsh" => return ShellType::Zsh,
            "fish" => return ShellType::Fish,
            _ => {}
        }
    }

    ShellType::Unknown
}

/// Get the shell configuration file path for the detected shell
pub fn get_shell_config_path(shell_type: &ShellType) -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("cannot get home directory"))?;

    let config_file = match shell_type {
        ShellType::Bash => {
            // Try .bashrc first, then .bash_profile
            let bashrc = home_dir.join(".bashrc");
            if bashrc.exists() {
                bashrc
            } else {
                home_dir.join(".bash_profile")
            }
        }
        ShellType::Zsh => home_dir.join(".zshrc"),
        ShellType::Fish => {
            let fish_config_dir = home_dir.join(".config").join("fish");
            if !fish_config_dir.exists() {
                fs::create_dir_all(&fish_config_dir)?;
            }
            fish_config_dir.join("config.fish")
        }
        ShellType::Unknown => return Err(anyhow!("unknown shell type")),
    };

    Ok(config_file)
}

/// Check if the PATH is already present in the shell config file
pub fn path_exists_in_config(config_path: &Path, bin_dir: &str) -> Result<bool> {
    if !config_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(config_path)?;

    // Check for the PATH in various common formats
    let patterns = vec![
        format!("export PATH=\"{}:$PATH\"", bin_dir),
        format!("export PATH='{}:$PATH'", bin_dir),
        format!("export PATH={}:$PATH", bin_dir),
        format!("set -gx PATH {} $PATH", bin_dir), // fish format
        format!("set PATH {} $PATH", bin_dir),     // fish format
    ];

    for pattern in patterns {
        if content.contains(&pattern) {
            return Ok(true);
        }
    }

    // Also check if the bin_dir is mentioned in PATH context (less strict)
    for line in content.lines() {
        if line.contains("PATH") && line.contains(bin_dir) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Add PATH to shell configuration file
pub fn add_path_to_config(shell_type: &ShellType, config_path: &Path, bin_dir: &str) -> Result<()> {
    // Create the file if it doesn't exist
    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(config_path, "")?;
    }

    let mut content = fs::read_to_string(config_path)?;

    // Add a newline at the end if the file doesn't end with one
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    // Add a comment and the PATH export
    let path_config = match shell_type {
        ShellType::Bash | ShellType::Zsh => {
            format!(
                "\n# Added by x - https://github.com/dwpeng/x\nexport PATH=\"{}:$PATH\"\n",
                bin_dir
            )
        }
        ShellType::Fish => {
            format!(
                "\n# Added by x - https://github.com/dwpeng/x\nset -gx PATH {} $PATH\n",
                bin_dir
            )
        }
        ShellType::Unknown => return Err(anyhow!("cannot add PATH for unknown shell type")),
    };

    content.push_str(&path_config);
    fs::write(config_path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_detect_shell_from_env() {
        // This test will detect the actual shell from environment
        let shell = detect_shell();
        // Just ensure it doesn't panic
        assert!(matches!(
            shell,
            ShellType::Bash | ShellType::Zsh | ShellType::Fish | ShellType::Unknown
        ));
    }

    #[test]
    fn test_path_exists_in_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config");

        // Test with non-existent file
        assert!(!path_exists_in_config(&config_path, "/test/bin").unwrap());

        // Test with file containing the PATH
        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(file, "export PATH=\"/test/bin:$PATH\"").unwrap();
        assert!(path_exists_in_config(&config_path, "/test/bin").unwrap());

        // Test with file not containing the PATH
        let config_path2 = temp_dir.path().join("test_config2");
        let mut file2 = fs::File::create(&config_path2).unwrap();
        writeln!(file2, "export PATH=\"/other/bin:$PATH\"").unwrap();
        assert!(!path_exists_in_config(&config_path2, "/test/bin").unwrap());
    }

    #[test]
    fn test_add_path_to_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_bashrc");

        // Test adding PATH to bash config
        add_path_to_config(&ShellType::Bash, &config_path, "/test/bin").unwrap();
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("export PATH=\"/test/bin:$PATH\""));
        assert!(content.contains("# Added by x"));

        // Test fish config
        let fish_config_path = temp_dir.path().join("test_config.fish");
        add_path_to_config(&ShellType::Fish, &fish_config_path, "/test/bin").unwrap();
        let content = fs::read_to_string(&fish_config_path).unwrap();
        assert!(content.contains("set -gx PATH /test/bin $PATH"));
    }
}
