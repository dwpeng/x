use x::cli::*;
use x::config::{Config, GLOBAL_DEFAULT_GROUP_NAME, get_config_path, load_config};
use x::confirm;
use x::process;

use clap::Parser;
use colored::Colorize;

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

    let group_name = cmd.group.unwrap_or(GLOBAL_DEFAULT_GROUP_NAME.to_string());

    let r = conf.find(&group_name, program).unwrap_or_else(|| {
        eprintln!(
            "Error: Program {} not found in group {}",
            program.red(),
            group_name.red()
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
    let exit_code = run.run_and_monitor();
    exit(exit_code.unwrap_or(1));
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

    let group_name = cmd.group.unwrap_or(GLOBAL_DEFAULT_GROUP_NAME.to_string());

    conf.add(&group_name, &cmd.path, cmd.name)
        .unwrap_or_else(|e| {
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
    println!("Added {} to group {}", cmd.path, group_name);
}

pub fn list(cmd: ListCommand) {
    let conf = load_config(false).unwrap_or_else(|e| {
        eprintln!("Error: cannot load config: {}", e);
        std::process::exit(1);
    });

    if cmd.all {
        conf.pretty_print(None);
    } else {
        conf.pretty_print(Some(&conf.active_group));
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

    println!("Add the following to your shell config file to use x");
    let bin_dir = conf.bin_dir.to_str().unwrap();
    let msg = format!("export PATH=\"{}:$PATH\"", bin_dir);
    println!("{}", msg.color(colored::Color::Green));
}

pub fn rm(cmd: RmCommand) {
    let mut conf = load_config(false).unwrap_or_else(|e| {
        eprintln!("Error: cannot load config: {}", e);
        std::process::exit(1);
    });

    let group_name = cmd.group.as_deref().unwrap_or(GLOBAL_DEFAULT_GROUP_NAME);

    // check if group exists
    if !conf.group_exists(group_name) {
        eprintln!("group {} does not exist", group_name.green());
        std::process::exit(1);
    }

    // double check from user before removing
    if cmd.delete {
        let message = if let Some(name) = &cmd.name {
            format!(
                "Are you sure you want to remove {} from group {}?",
                name.green(),
                group_name.green()
            )
        } else {
            format!(
                "Are you sure you want to remove all executables from group {}?",
                group_name.green()
            )
        };
        if !confirm(&message) {
            return;
        }
    }

    conf.remove(group_name, cmd.name.as_deref(), cmd.delete)
        .unwrap_or_else(|e| {
            eprintln!("Error: cannot remove executable: {}", e);
            std::process::exit(1);
        });

    conf.save(get_config_path().unwrap()).unwrap_or_else(|e| {
        eprintln!("Error: cannot save config: {}", e);
        std::process::exit(1);
    });
}

pub fn switch(cmd: SwitchCommand) {
    let mut conf = load_config(false).unwrap_or_else(|e| {
        eprintln!("Error: cannot load config: {}", e);
        std::process::exit(1);
    });

    let need_active_group = cmd.group.unwrap_or(GLOBAL_DEFAULT_GROUP_NAME.to_owned());

    conf.switch(&need_active_group).unwrap_or_else(|e| {
        eprintln!("Error: cannot switch group: {}", e);
        std::process::exit(1);
    });

    conf.save(get_config_path().unwrap()).unwrap_or_else(|e| {
        eprintln!("Error: cannot save config: {}", e);
        std::process::exit(1);
    });
    println!("Switched to group {}", conf.active_group.green());
}

pub fn rename(cmd: RenameCommand) {
    let mut conf = load_config(false).unwrap_or_else(|e| {
        eprintln!("Error: cannot load config: {}", e);
        std::process::exit(1);
    });

    let group_name = cmd.group.as_deref().unwrap_or(GLOBAL_DEFAULT_GROUP_NAME);

    conf.rename(group_name, &cmd.old_name, &cmd.new_name)
        .unwrap_or_else(|e| {
            eprintln!("Error: cannot rename executable: {}", e);
            std::process::exit(1);
        });

    conf.save(get_config_path().unwrap()).unwrap_or_else(|e| {
        eprintln!("Error: cannot save config: {}", e);
        std::process::exit(1);
    });

    println!(
        "Renamed {} to {} in group {}",
        cmd.old_name.green(),
        cmd.new_name.green(),
        group_name.cyan()
    );
}

pub fn info(cmd: InfoCommand) {
    let conf = load_config(false).unwrap_or_else(|e| {
        eprintln!("Error: cannot load config: {}", e);
        std::process::exit(1);
    });

    let group_name = cmd.group.as_deref().unwrap_or(GLOBAL_DEFAULT_GROUP_NAME);

    let bin = conf.get_bin_info(group_name, &cmd.name).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    println!("{}", "Executable Information".bold().cyan());
    println!("  {}: {}", "Name".bold(), bin.name.green());
    println!("  {}: {}", "Path".bold(), bin.path.display().to_string().green());
    println!("  {}: {}", "Group".bold(), group_name.cyan());
    println!("  {}: {}", "Status".bold(), 
        if bin.enabled { "enabled".green() } else { "disabled".red() });
    
    if let Some(source_dir) = &bin.source_dir {
        println!("  {}: {}", "Source Directory".bold(), source_dir.display().to_string().yellow());
    }
    
    println!("  {}: {}", "In Active Group".bold(),
        if conf.active_group == group_name { "yes".green() } else { "no".yellow() });
    
    // Check if file exists
    let exists = bin.path.exists();
    println!("  {}: {}", "File Exists".bold(), 
        if exists { "yes".green() } else { "no".red() });
}

pub fn enable(cmd: EnableCommand) {
    let mut conf = load_config(false).unwrap_or_else(|e| {
        eprintln!("Error: cannot load config: {}", e);
        std::process::exit(1);
    });

    let group_name = cmd.group.as_deref().unwrap_or(GLOBAL_DEFAULT_GROUP_NAME);

    conf.set_enabled(group_name, &cmd.name, true)
        .unwrap_or_else(|e| {
            eprintln!("Error: cannot enable executable: {}", e);
            std::process::exit(1);
        });

    conf.save(get_config_path().unwrap()).unwrap_or_else(|e| {
        eprintln!("Error: cannot save config: {}", e);
        std::process::exit(1);
    });

    println!("Enabled {} in group {}", cmd.name.green(), group_name.cyan());
}

pub fn disable(cmd: DisableCommand) {
    let mut conf = load_config(false).unwrap_or_else(|e| {
        eprintln!("Error: cannot load config: {}", e);
        std::process::exit(1);
    });

    let group_name = cmd.group.as_deref().unwrap_or(GLOBAL_DEFAULT_GROUP_NAME);

    conf.set_enabled(group_name, &cmd.name, false)
        .unwrap_or_else(|e| {
            eprintln!("Error: cannot disable executable: {}", e);
            std::process::exit(1);
        });

    conf.save(get_config_path().unwrap()).unwrap_or_else(|e| {
        eprintln!("Error: cannot save config: {}", e);
        std::process::exit(1);
    });

    println!("Disabled {} in group {}", cmd.name.green(), group_name.cyan());
}

pub fn search(cmd: SearchCommand) {
    let conf = load_config(false).unwrap_or_else(|e| {
        eprintln!("Error: cannot load config: {}", e);
        std::process::exit(1);
    });

    let results = conf.search(&cmd.query);

    if results.is_empty() {
        println!("No executables found matching '{}'", cmd.query.yellow());
        return;
    }

    println!("Found {} executable(s) matching '{}':", results.len(), cmd.query.yellow());
    for (group_name, bin_name, bin) in results {
        let status = if bin.enabled { "" } else { " [disabled]" };
        let active = if conf.active_group == group_name { "*" } else { " " };
        println!(
            "  {} {} / {} -> {}{}",
            active.green().bold(),
            group_name.cyan(),
            bin_name.green(),
            bin.path.display().to_string().green(),
            status.red()
        );
    }
}

pub static AVAILABLE_SUBCOMMANDS: &[&str] =
    &["run", "r", "add", "rm", "list", "ls", "init", "s", "switch", "rename", "info", "enable", "disable", "search"];

fn main() {
    let mut args = std::env::args();
    if let Some(first_arg) = args.nth(1)
        && !AVAILABLE_SUBCOMMANDS.contains(&first_arg.as_str()) {
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
        Commands::Switch(s) => switch(s),
        Commands::Rename(r) => rename(r),
        Commands::Info(i) => info(i),
        Commands::Enable(e) => enable(e),
        Commands::Disable(d) => disable(d),
        Commands::Search(s) => search(s),
    }
}
