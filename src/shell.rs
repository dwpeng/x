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
    get_shell_config_path_from_home(shell_type, &home_dir)
}

fn get_shell_config_path_from_home(shell_type: &ShellType, home_dir: &Path) -> Result<PathBuf> {
    let config_file = match shell_type {
        ShellType::Bash => home_dir.join(".bashrc"),
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

    // Single-pass: check each line for a PATH export/set that references bin_dir.
    for line in content.lines() {
        let trimmed = line.trim();
        if (trimmed.starts_with("export PATH")
            || trimmed.starts_with("set -gx PATH")
            || trimmed.starts_with("set PATH"))
            && trimmed.contains(bin_dir)
        {
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

fn path_contains_dir(path_var: &std::ffi::OsStr, dir: &Path) -> bool {
    let target = dir.canonicalize().ok();
    env::split_paths(path_var).any(|entry| {
        if entry == dir {
            return true;
        }

        if let (Some(target), Ok(entry_canonical)) = (&target, entry.canonicalize()) {
            return *target == entry_canonical;
        }

        false
    })
}

pub fn is_dir_in_current_path(dir: &Path) -> bool {
    env::var_os("PATH")
        .map(|path_var| path_contains_dir(path_var.as_os_str(), dir))
        .unwrap_or(false)
}

fn copy_executable_to_local_x(current_exe: &Path, home_dir: &Path) -> Result<PathBuf> {
    let exe_name = current_exe
        .file_name()
        .ok_or_else(|| anyhow!("cannot get executable filename"))?;
    let dest_dir = home_dir.join(".local").join("x");
    fs::create_dir_all(&dest_dir)?;
    let dest_path = dest_dir.join(exe_name);
    fs::copy(current_exe, &dest_path)?;
    Ok(dest_path)
}

fn maybe_copy_executable_if_dir_not_in_path(
    current_exe: &Path,
    path_var: Option<&std::ffi::OsStr>,
    home_dir: &Path,
) -> Result<Option<PathBuf>> {
    let current_dir = current_exe
        .parent()
        .ok_or_else(|| anyhow!("cannot get executable directory"))?;

    let in_path = path_var
        .map(|path| path_contains_dir(path, current_dir))
        .unwrap_or(false);
    if in_path {
        return Ok(None);
    }

    let copied_path = copy_executable_to_local_x(current_exe, home_dir)?;
    Ok(Some(copied_path))
}

pub fn maybe_copy_current_executable_to_local_x() -> Result<Option<PathBuf>> {
    let current_exe = env::current_exe()?;
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("cannot get home directory"))?;
    maybe_copy_executable_if_dir_not_in_path(&current_exe, env::var_os("PATH").as_deref(), &home_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
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

    #[test]
    fn test_get_shell_config_path_from_home_uses_shell_type_not_file_existence() {
        let temp_dir = TempDir::new().unwrap();
        let home_dir = temp_dir.path();

        fs::write(home_dir.join(".bash_profile"), "# existing\n").unwrap();
        let bash_path = get_shell_config_path_from_home(&ShellType::Bash, home_dir).unwrap();
        assert_eq!(bash_path, home_dir.join(".bashrc"));

        let zsh_path = get_shell_config_path_from_home(&ShellType::Zsh, home_dir).unwrap();
        assert_eq!(zsh_path, home_dir.join(".zshrc"));
    }

    #[test]
    fn test_path_contains_dir() {
        let temp_dir = TempDir::new().unwrap();
        let in_path = temp_dir.path().join("in_path");
        fs::create_dir_all(&in_path).unwrap();
        let not_in_path = temp_dir.path().join("not_in_path");
        fs::create_dir_all(&not_in_path).unwrap();

        let path_var = env::join_paths([PathBuf::from("/usr/bin"), in_path.clone()]).unwrap();
        assert!(path_contains_dir(path_var.as_os_str(), &in_path));
        assert!(!path_contains_dir(path_var.as_os_str(), &not_in_path));
    }

    #[test]
    fn test_copy_executable_to_local_x() {
        let temp_dir = TempDir::new().unwrap();
        let home_dir = temp_dir.path().join("home");
        fs::create_dir_all(&home_dir).unwrap();

        let exe_path = temp_dir.path().join("x");
        fs::write(&exe_path, b"#!/bin/sh\necho test\n").unwrap();

        let copied_path = copy_executable_to_local_x(&exe_path, &home_dir).unwrap();
        assert_eq!(copied_path, home_dir.join(".local").join("x").join("x"));
        assert!(copied_path.exists());
        assert_eq!(fs::read(&copied_path).unwrap(), fs::read(&exe_path).unwrap());
    }

    #[test]
    fn test_maybe_copy_executable_if_dir_not_in_path() {
        let temp_dir = TempDir::new().unwrap();
        let home_dir = temp_dir.path().join("home");
        fs::create_dir_all(&home_dir).unwrap();

        let exe_dir = temp_dir.path().join("bin");
        fs::create_dir_all(&exe_dir).unwrap();
        let exe_path = exe_dir.join("x");
        fs::write(&exe_path, b"#!/bin/sh\necho test\n").unwrap();

        let path_without_exe_dir = env::join_paths([PathBuf::from("/usr/bin")]).unwrap();
        let copied = maybe_copy_executable_if_dir_not_in_path(
            &exe_path,
            Some(path_without_exe_dir.as_os_str()),
            &home_dir,
        )
        .unwrap();
        assert_eq!(copied, Some(home_dir.join(".local").join("x").join("x")));

        let path_with_exe_dir = env::join_paths([exe_dir]).unwrap();
        let skipped = maybe_copy_executable_if_dir_not_in_path(
            &exe_path,
            Some(path_with_exe_dir.as_os_str()),
            &home_dir,
        )
        .unwrap();
        assert_eq!(skipped, None);
    }
}
