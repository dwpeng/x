use colored::Colorize;
use x::cli::*;
use x::config::{Config, get_config_path, load_config};
use x::process;

use clap::Parser;

use std::path::Path;
use std::process::exit;

pub fn run(cmd: RunCommand) {
    let conf_path = get_config_path().unwrap_or_else(|e| {
        eprintln!("Error: cannot get config path: {}", e);
        exit(1);
    });

    if !conf_path.exists() {
        eprintln!(
            "x is not configured yet. Please run `{}` first.",
            "x init".green()
        );
        exit(1);
    }

    let conf = Config::load(&conf_path).unwrap_or_else(|e| {
        eprintln!(
            "Error: cannot load config file {}: {}",
            conf_path.display(),
            e
        );
        exit(1);
    });

    let args = cmd.args.unwrap_or_else(|| {
        eprintln!("Error: No program specified");
        exit(1);
    });

    if args.is_empty() || args[0].is_empty() {
        eprintln!("Error: No program specified");
        exit(1);
    }

    let program = &args[0];
    let args = &args[1..];

    if Path::new(program).is_absolute() {
        if !Path::new(program).exists() {
            eprintln!("Error: Program path {} does not exist", program.red());
            exit(1);
        }
        run_and_monitor(program, args);
        return;
    }

    let r = conf.find(&cmd.group, program).unwrap_or_else(|| {
        eprintln!(
            "Error: Program {} not found in group {}",
            program.red(),
            cmd.group.red()
        );
        exit(1);
    });

    if !Path::new(&r.path).exists() {
        eprintln!(
            "Error: Program path {} does not exist",
            r.path.to_str().unwrap().green()
        );
        exit(1);
    }

    let path = r.path.to_str().unwrap();
    run_and_monitor(path, args);
}

fn run_and_monitor(program: &str, args: &[String]) {
    let run = process::Run::new(program, args);
    let stats = run.run_and_monitor();
    println!("{}", stats);
    exit(stats.exit_code.unwrap_or(1));
}

pub fn add(cmd: AddCommand) {
    let conf_path = match get_config_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: cannot get config path: {}", e);
            std::process::exit(1);
        }
    };

    // check if the config file exists
    if !conf_path.exists() {
        let conf = Config::default();
        conf.save(&conf_path).unwrap_or_else(|e| {
            eprintln!(
                "Error: cannot create config file {}: {}",
                conf_path.display(),
                e
            );
            std::process::exit(1);
        });
    }

    let mut conf = match Config::load(&conf_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Error: cannot load config file {}: {}",
                conf_path.display(),
                e
            );
            std::process::exit(1);
        }
    };

    if cmd.path.is_empty() {
        eprintln!("Error: No path specified");
        std::process::exit(1);
    }
    conf.add(&cmd.group, &cmd.path, cmd.name).unwrap_or_else(|e| {
        eprintln!("Error: cannot add path {}: {}", cmd.path, e);
        std::process::exit(1);
    });

    conf.save(&conf_path).unwrap_or_else(|e| {
        eprintln!(
            "Error: cannot save config file {}: {}",
            conf_path.display(),
            e
        );
        std::process::exit(1);
    });
    println!("Added {} to group {}", cmd.path, cmd.group);
}

pub fn list(cmd: ListCommand) {
    let conf = load_config(false).unwrap_or_else(|e| {
        eprintln!("Error: cannot load config: {}", e);
        std::process::exit(1);
    });

    if cmd.all {
        conf.pretty_print(None);
    } else {
        conf.pretty_print(Some(&cmd.group));
    }
}

pub fn init(cmd: InitCommand) {
    let conf_path = match get_config_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: cannot get config path: {}", e);
            std::process::exit(1);
        }
    };

    if conf_path.exists() && !cmd.force {
        eprintln!(
            "Error: config file {} already exists. Use `-f` to overwrite.",
            conf_path.display()
        );
        std::process::exit(1);
    }

    if conf_path.exists() && cmd.force {
        let backup_path = conf_path.with_extension("bak");
        std::fs::copy(&conf_path, &backup_path).unwrap_or_else(|e| {
            eprintln!(
                "Error: cannot backup existing config file {}: {}",
                conf_path.display(),
                e
            );
            std::process::exit(1);
        });
    }

    let conf = Config::default();
    conf.save(&conf_path).unwrap_or_else(|e| {
        eprintln!(
            "Error: cannot create config file {}: {}",
            conf_path.display(),
            e
        );
        std::process::exit(1);
    });
    println!(
        "Created config file at {}",
        conf_path.display().to_string().color(colored::Color::Green)
    );
}

pub fn rm(cmd: RmCommand) {
    let mut conf = load_config(false).unwrap_or_else(|e| {
        eprintln!("Error: cannot load config: {}", e);
        std::process::exit(1);
    });

    conf.remove(&cmd.group, cmd.name.as_deref())
        .unwrap_or_else(|e| {
            eprintln!("Error: cannot remove executable: {}", e);
            std::process::exit(1);
        });

    conf.save(&get_config_path().unwrap()).unwrap_or_else(|e| {
        eprintln!("Error: cannot save config: {}", e);
        std::process::exit(1);
    });
}

pub static AVALIABLE_SUBCOMMANDS: &'static [&'static str] =
    &["run", "r", "add", "rm", "list", "init"];

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() >= 2 && !AVALIABLE_SUBCOMMANDS.contains(&args[1].as_str()) {
        let run_cmd = RunCommand::parse();
        run(run_cmd);
        return;
    }

    let cli = Cli::parse();
    match cli.command {
        Commands::Run(r) => run(r),
        Commands::Add(a) => add(a),
        Commands::List(l) => list(l),
        Commands::Init(i) => init(i),
        Commands::Rm(r) => rm(r),
    }
}
