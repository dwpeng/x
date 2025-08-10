use colored::Colorize;
use psutil::process::Process as PsutilProcess;
use std::fmt::Display;
use std::process::Command;
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

pub struct Run<'a> {
    pub command: &'a str,
    pub args: &'a [String],
}

impl<'a> Run<'a> {
    pub fn new(command: &'a str, args: &'a [String]) -> Self {
        Run { command, args }
    }

    pub fn run_and_monitor(&self) -> ProcessStats {
        let start_time = Instant::now();
        let child = Command::new(self.command)
            .args(self.args)
            .spawn()
            .expect("Failed to run command");

        let pid = child.id();
        let max_rss_b = thread::spawn(move || {
            let mut max_rss = 0;
            let mut proc = PsutilProcess::new(pid);
            while let Ok(p) = &mut proc {
                if p.is_running() {
                    if let Ok(mem) = p.memory_info() {
                        let rss = mem.rss();
                        if rss > max_rss {
                            max_rss = rss;
                        }
                    }
                    sleep(Duration::from_millis(100));
                } else {
                    break;
                }
            }
            max_rss
        });

        let status = child
            .wait_with_output()
            .expect("Failed to wait for child process");
        let max_rss_b = max_rss_b.join().unwrap();

        ProcessStats {
            duration: start_time.elapsed().as_secs_f64(),
            max_rss_b,
            exit_code: status.status.code(),
        }
    }
}

pub fn green<T: ToString + 'static>(text: T) -> colored::ColoredString {
    // if floating point, show two decimal places
    if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f64>()
        || std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>()
    {
        let text = format!("{:.2}", text.to_string().parse::<f64>().unwrap_or(0.0));
        text.green()
    } else {
        text.to_string().green()
    }
}

pub struct ProcessStats {
    pub duration: f64,
    pub max_rss_b: u64,
    pub exit_code: Option<i32>,
}

impl Display for ProcessStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seconds = self.duration as u64;
        if seconds > 60 {
            write!(
                f,
                "Duration: {} minutes {} seconds, ",
                green(seconds / 60),
                green(seconds % 60)
            )?;
        } else {
            write!(f, "Duration: {} seconds, ", green(self.duration))?;
        }

        let max_rss_display = if self.max_rss_b > 1024 * 1024 {
            format!("{} MB", green(self.max_rss_b as f64 / 1024.0 / 1024.0))
        } else if self.max_rss_b > 1024 {
            format!("{} KB", green(self.max_rss_b as f64 / 1024.0))
        } else {
            format!("{} B", green(self.max_rss_b as f64))
        };

        write!(f, "Max RSS: {}", max_rss_display)
    }
}
