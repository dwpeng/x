use std::path::Path;
use std::process::Command;

pub struct Run<'a> {
    pub command: &'a str,
    pub args: &'a [String],
}

impl<'a> Run<'a> {
    pub fn new(command: &'a str, args: &'a [String]) -> Self {
        Run { command, args }
    }

    pub fn run_and_monitor(&self) -> Option<i32> {
        let mut child = Command::new(self.command)
            .args(self.args)
            .spawn()
            .ok()
            .or_else(|| self.spawn_script_with_interpreter())?;
        child.wait().ok()?.code()
    }

    fn spawn_script_with_interpreter(&self) -> Option<std::process::Child> {
        let extension = Path::new(self.command)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())?;

        match extension.as_str() {
            "sh" => Command::new("bash")
                .arg(self.command)
                .args(self.args)
                .spawn()
                .ok(),
            "py" => {
                #[cfg(windows)]
                let python_cmd = "python";
                #[cfg(not(windows))]
                let python_cmd = "python3";

                Command::new(python_cmd)
                    .arg(self.command)
                    .args(self.args)
                    .spawn()
                    .ok()
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Run;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn run_and_monitor_returns_none_when_command_missing() {
        let args: Vec<String> = vec![];
        let run = Run::new("x_command_should_not_exist_12345", &args);
        assert_eq!(run.run_and_monitor(), None);
    }

    #[test]
    fn run_and_monitor_returns_status_for_valid_command() {
        #[cfg(unix)]
        let (command, args) = ("true", vec![]);

        #[cfg(windows)]
        let (command, args) = (
            "cmd",
            vec!["/C".to_string(), "exit".to_string(), "0".to_string()],
        );

        let run = Run::new(command, &args);
        assert_eq!(run.run_and_monitor(), Some(0));
    }

    #[test]
    #[cfg(unix)]
    fn run_non_executable_shell_script_with_bash_fallback() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let script_path = temp_dir.path().join("script.sh");
        fs::write(&script_path, "exit 0\n").expect("failed to write script");

        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o644))
            .expect("failed to set script permissions");

        let args = vec![];
        let run = Run::new(script_path.to_str().expect("script path is not utf-8"), &args);
        assert_eq!(run.run_and_monitor(), Some(0));
    }
}
