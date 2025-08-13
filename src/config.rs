use anyhow::{Result, anyhow};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Bin {
    pub path: PathBuf,
    #[serde(default = "default_true")]
    pub active: bool,
    pub source_dir: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Group {
    pub index: usize,
    pub bins: HashMap<String, Bin>,
    pub active: bool,
}

impl Default for Group {
    fn default() -> Self {
        Group {
            index: 0,
            bins: HashMap::new(),
            active: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub default_group: Option<String>,
    pub groups: HashMap<String, Group>,
}

pub static GLOBAL_DEFAULT_GROUP_NAME: &'static str = "global-default-group-name";

impl Default for Config {
    fn default() -> Self {
        Config {
            default_group: Some(GLOBAL_DEFAULT_GROUP_NAME.to_string()),
            groups: HashMap::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

pub fn get_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("cannot get home dir"))?;
    let config_dir = home_dir.join(".config").join("x");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    Ok(config_dir.join("config.json"))
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn add(
        &mut self,
        group: impl Into<String>,
        path: impl AsRef<Path>,
        name: Option<String>,
    ) -> Result<()> {
        let path = path.as_ref();
        let group_name = group.into();

        if path.is_file() && is_executable(path) {
            let bin_name = if name.is_none() {
                executable_name(path)?
            } else {
                name.unwrap()
            };
            let bin = Bin {
                path: path.to_path_buf().canonicalize().unwrap(),
                active: true,
                source_dir: None,
            };
            let g = self.groups.entry(group_name).or_insert_with(Group::default);
            g.bins.insert(bin_name, bin);
            g.index = g.bins.len() - 1;
            return Ok(());
        }

        if path.is_dir() {
            let g = self.groups.entry(group_name).or_insert_with(Group::default);
            g.active = true;
            g.index = g.bins.len() - 1;
            for (name, file_path) in expand_dir(path)? {
                g.bins.insert(
                    name,
                    Bin {
                        path: file_path,
                        active: true,
                        source_dir: Some(path.to_path_buf()),
                    },
                );
            }
            return Ok(());
        }

        anyhow::bail!("path is neither an executable file nor a directory")
    }

    pub fn pretty_print(&self, group: Option<&str>) {
        let mut groups: Vec<(&String, &Group)> = self.groups.iter().collect();
        groups.sort_by(|a, b| a.1.index.cmp(&b.1.index));
        for (gn, g) in groups {
            if let Some(filter) = group {
                if filter != gn {
                    continue;
                }
            }

            let g_color = if g.active {
                gn.green().bold()
            } else {
                gn.color(Color::TrueColor { r: 6, g: 6, b: 6 })
            };
            println!("{}", g_color);

            if !g.active {
                continue;
            }

            let mut count = 1;
            for (bn, b) in &g.bins {
                let color = if b.active {
                    Color::Green
                } else {
                    Color::TrueColor { r: 6, g: 6, b: 6 }
                };
                println!(
                    " {:2}. {} -> {}",
                    count,
                    bn.color(color),
                    b.path.display().to_string().color(color),
                );
                count += 1;
            }
        }
    }

    pub fn remove(&mut self, group: &str, name: Option<&str>, delete: bool) -> Result<()> {
        let mut delete_group = false;
        if let Some(g) = self.groups.get_mut(group) {
            if let Some(n) = name {
                if let Some(b) = g.bins.get_mut(n) {
                    b.active = false;
                }
                if delete {
                    g.bins.remove(n);
                }
            } else {
                if delete {
                    delete_group = true;
                    g.bins.clear();
                }
                g.active = false;
                for b in g.bins.values_mut() {
                    b.active = false;
                }
            }
        }
        if delete_group {
            self.groups.remove(group);
        }
        Ok(())
    }

    pub fn activate(&mut self, group: &str, name: Option<&str>) -> Result<()> {
        if let Some(g) = self.groups.get_mut(group) {
            if let Some(n) = name {
                if let Some(b) = g.bins.get_mut(n) {
                    b.active = true;
                }
            } else {
                g.active = true;
                for b in g.bins.values_mut() {
                    b.active = true;
                }
            }
        }
        Ok(())
    }

    pub fn find(&self, group: &str, name: &str) -> Option<&Bin> {
        // check if active first
        if let Some(g) = self.groups.get(group) {
            if !g.active {
                return None;
            }
            if let Some(b) = g.bins.get(name) {
                if b.active {
                    return Some(b);
                } else {
                    return None;
                }
            }
        }
        None
    }

    pub fn group_exists(&self, group: &str) -> bool {
        self.groups.contains_key(group)
    }
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = p.metadata() {
        meta.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

#[cfg(windows)]
fn is_executable(p: &Path) -> bool {
    p.extension()
        .and_then(|s| s.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "exe" | "bat" | "cmd" | "ps1" | "com"
            )
        })
        .unwrap_or(false)
}

fn expand_dir(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut res = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_file() {
            continue;
        }
        let path = entry.path();
        if is_executable(&path) {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow!("non-utf8 filename"))?;
            res.push((name, path.canonicalize()?));
        }
    }
    Ok(res)
}

fn executable_name(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .ok_or_else(|| anyhow!("cannot extract file stem"))?
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 filename"))?;
    Ok(stem.into())
}

pub fn load_config(create: bool) -> Result<Config> {
    let conf_path = get_config_path()?;
    if !conf_path.exists() {
        if create {
            let c = Config::default();
            c.save(&conf_path)?;
            Ok(c)
        } else {
            anyhow::bail!(
                "config file does not exist, please run `{}` first",
                "x init".bold().green()
            );
        }
    } else {
        Config::load(&conf_path)
    }
}
