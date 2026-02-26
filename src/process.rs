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
        let mut child = Command::new(self.command).args(self.args).spawn().ok()?;
        child.wait().ok()?.code()
    }
}

#[cfg(test)]
mod tests {
    use super::Run;

    #[test]
    fn run_and_monitor_returns_none_when_command_missing() {
        let args: Vec<String> = vec![];
        let run = Run::new("x_command_should_not_exist_12345", &args);
        assert_eq!(run.run_and_monitor(), None);
    }

    #[test]
    fn run_and_monitor_returns_status_for_valid_command() {
        let args = vec!["--version".to_string()];
        let run = Run::new("rustc", &args);
        assert!(run.run_and_monitor().is_some());
    }
}
