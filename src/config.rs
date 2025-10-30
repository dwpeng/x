use anyhow::{Result, anyhow};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Bin {
    pub name: String,
    pub path: PathBuf,
    pub source_dir: Option<PathBuf>,
}

impl Bin {
    pub fn install(&self, dir_path: &Path) -> Result<()> {
        if dir_path.join(self.name.as_str()).exists() {
            fs::remove_file(dir_path.join(self.name.as_str()))?;
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&self.path, dir_path.join(self.name.as_str()))?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&self.path, dir_path.join(self.name.as_str()))?;

        Ok(())
    }

    pub fn uninstall(&self, dir_path: &Path) -> Result<()> {
        if dir_path.join(self.name.as_str()).exists() {
            fs::remove_file(dir_path.join(self.name.as_str()))?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Group {
    pub index: usize,
    pub bins: HashMap<String, Bin>,
}

impl Group {
    pub fn remove_bin_by_name(&mut self, name: &str, bin_dir: &Path) -> Result<()> {
        if let Some(bin) = self.bins.get(name) {
            bin.uninstall(bin_dir)?;
            self.bins.remove(name);
        }
        Ok(())
    }

    pub fn remove_bin_by_path(&mut self, path: &PathBuf, bin_dir: &Path) -> Result<()> {
        let to_remove: Vec<_> = self.bins.iter()
            .filter_map(|(name, bin)| {
                bin.source_dir.as_ref()
                    .filter(|source_dir| *source_dir == path)
                    .map(|_| name.clone())
            })
            .collect();

        for name in to_remove {
            if let Some(bin) = self.bins.get(&name) {
                bin.uninstall(bin_dir)?;
            }
            self.bins.remove(&name);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub active_group: String,
    pub bin_dir: PathBuf,
    pub groups: HashMap<String, Group>,
}

pub static GLOBAL_DEFAULT_GROUP_NAME: &str = "global-default-group-name";

pub fn get_bin_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("cannot get home dir"))?;
    let bin_dir = home_dir.join(".local").join("bin").join("x");
    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir)?;
    }
    Ok(bin_dir)
}

impl Default for Config {
    fn default() -> Self {
        let bin_dir = get_bin_dir();
        if bin_dir.is_err() {
            panic!("cannot get bin dir")
        }
        let bin_dir = bin_dir.unwrap();
        Config {
            active_group: GLOBAL_DEFAULT_GROUP_NAME.to_string(),
            bin_dir,
            groups: HashMap::new(),
        }
    }
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
            let bin_name = if let Some(name) = name {
                name
            } else {
                executable_name(path)?
            };
            let bin = Bin {
                name: bin_name.clone(),
                path: path.to_path_buf().canonicalize().unwrap(),
                source_dir: None,
            };

            if self.active_group == group_name {
                bin.install(&self.bin_dir)?;
            }

            let g = self.groups.entry(group_name).or_default();
            g.bins.insert(bin_name, bin);
            g.index = g.bins.len() - 1;
            return Ok(());
        }

        if path.is_dir() {
            let g = self
                .groups
                .entry(group_name.clone())
                .or_default();
            g.index = g.bins.len() - 1;
            for (name, file_path) in collect_executables_from_dir(path)? {
                let bin = Bin {
                    name: name.clone(),
                    path: file_path,
                    source_dir: Some(path.to_path_buf().canonicalize().unwrap()),
                };
                if self.active_group == group_name {
                    bin.install(&self.bin_dir)?;
                }
                g.bins.insert(name, bin);
            }
            return Ok(());
        }

        anyhow::bail!("path is neither an executable file nor a directory")
    }

    pub fn pretty_print(&self, group: Option<&str>) {
        let mut groups: Vec<(&String, &Group)> = self.groups.iter().collect();
        groups.sort_by(|a, b| a.0.cmp(b.0));

        for (gn, g) in groups {
            if let Some(filter) = group
                && filter != gn {
                    continue;
                }

            if gn == &self.active_group {
                println!("{} {}", "*".green().bold(), gn.cyan().bold());
            } else {
                println!("  {}", gn.cyan());
            }
            let mut count = 1;
            for (bn, b) in &g.bins {
                println!(
                    "  {:2}. {} -> {}",
                    count,
                    bn.color(Color::Green),
                    b.path.display().to_string().color(Color::Green),
                );
                count += 1;
            }
        }
    }

    pub fn remove(&mut self, group: &str, name: Option<&str>, delete: bool) -> Result<()> {
        let mut delete_group = false;
        if let Some(g) = self.groups.get_mut(group) {
            if let Some(name) = name {
                if name.contains("/") || name.contains("\\") {
                    // is path
                    // try to find path in bins
                    let path = PathBuf::from(name);
                    g.remove_bin_by_path(&path, &self.bin_dir)?;
                } else {
                    g.remove_bin_by_name(name, &self.bin_dir)?;
                }
            } else if delete {
                delete_group = true;
                for (_, bin) in g.bins.iter() {
                    bin.uninstall(&self.bin_dir)?;
                }
                g.bins.clear();
            }
        }
        if delete_group {
            self.groups.remove(group);
        }
        Ok(())
    }

    pub fn find(&self, group: &str, name: &str) -> Option<&Bin> {
        if let Some(g) = self.groups.get(group)
            && let Some(b) = g.bins.get(name) {
                return Some(b);
            }
        None
    }

    pub fn switch(&mut self, need_active_group_name: &str) -> Result<()> {
        if self.active_group == need_active_group_name {
            return Ok(());
        }

        if !self.group_exists(need_active_group_name) {
            anyhow::bail!("group {} does not exist", need_active_group_name);
        }

        let old_group_name = &self.active_group;
        let bin_dir = &self.bin_dir;
        let old_groups = self.groups.get_mut(old_group_name).unwrap();
        for (_, b) in old_groups.bins.iter() {
            b.uninstall(bin_dir)?;
        }
        let new_groups = self.groups.get_mut(need_active_group_name).unwrap();
        for (_, b) in new_groups.bins.iter() {
            b.install(bin_dir)?;
        }
        self.active_group = need_active_group_name.to_string();
        Ok(())
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

fn collect_executables_from_dir(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
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
