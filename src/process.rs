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
        let child = Command::new(self.command)
            .args(self.args)
            .spawn()
            .expect("Failed to run command");

        let status = child
            .wait_with_output()
            .expect("Failed to wait for child process");

        status.status.code()
    }
}
