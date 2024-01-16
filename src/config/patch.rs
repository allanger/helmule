use std::{
    fs::{self, read_dir, remove_dir_all, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::helpers::cli::{cli_exec, cli_exec_from_dir};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct RegexpPatch {
    pub(crate) path: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct GitPatch {
    path: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) enum YqOperations {
    Add,
    Delete,
    Replace,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct YqPatch {
    file: String,
    op: YqOperations,
    key: String,
    value: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct CustomCommandPatch {
    commands: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Patch {
    name: Option<String>,
    regexp: Option<RegexpPatch>,
    git: Option<GitPatch>,
    custom_command: Option<CustomCommandPatch>,
    yq: Option<YqPatch>,
}

impl Patch {
    pub(crate) fn apply(&self, chart_local_path: String, global_patches: Option<Vec<Patch>>) -> Result<(), Box<dyn std::error::Error>> {
        let patch_action: Box<dyn PatchInterface>;
        if self.is_ref(){
            let patch_ref = self.get_ref(global_patches)?;
            patch_action = Box::from(patch_action_from_definition(patch_ref)?);
        } else {
            patch_action = Box::from(patch_action_from_definition(self.clone())?);
        }
        patch_action.apply(chart_local_path)
    }
    pub(crate) fn get_path(&self) -> String {
        match patch_action_from_definition(self.clone()) {
            Ok(patch) => patch.get_path(),
            Err(_) => "".to_string(),
        }
    }
    pub(crate) fn set_path(&mut self, path: String) {
        if let Some(ref mut regexp) = self.regexp {
            regexp.path = path;
        } else if let Some(ref mut git) = self.git {
            git.path = path;
        }
    }

    fn is_ref(&self) -> bool {
        self.regexp.is_none()
            && self.git.is_none()
            && self.custom_command.is_none()
            && self.yq.is_none()
            && self.name.is_some()
    }

    pub(crate) fn get_ref(
        &self,
        global_patches: Option<Vec<Patch>>,
    ) -> Result<Patch, Box<dyn std::error::Error>> {
        match global_patches {
            Some(patches) => {
                let patch = patches
                    .iter()
                    .find(|&patch| patch.clone().name.unwrap() == self.clone().name.unwrap());
                match patch {
                    Some(patch) => {
                        return Ok(patch.clone());
                    }
                    None => {
                        return Err(Box::from(format!(
                            "global patch is not found: {}",
                            self.clone().name.unwrap()
                        )))
                    }
                }
            }
            None => {
                return Err(Box::from(format!(
                    "patch {} is recognized as a reference, but global patches are not defined",
                    self.clone().name.unwrap()
                )))
            }
        }
    }
}

trait PatchInterface {
    fn apply(&self, chart_local_path: String) -> Result<(), Box<dyn std::error::Error>>;
    fn get_path(&self) -> String;
    fn set_path(&mut self, new_path: String);
}

impl PatchInterface for YqPatch {
    fn apply(&self, chart_local_path: String) -> Result<(), Box<dyn std::error::Error>> {
        let cmd = match self.op {
            YqOperations::Add => {
                let value = match self
                    .value
                    .clone()
                    .unwrap()
                    .starts_with(['{', '[', '\"', '\''])
                {
                    true => self.value.clone().unwrap(),
                    false => format!("\"{}\"", self.value.clone().unwrap()),
                };
                format!("yq -i '{} += {}' {}", self.key, value, self.file)
            }
            YqOperations::Delete => format!("yq -i \'del({})\' {}", self.key, self.file),
            YqOperations::Replace => {
                let value = match self.value.clone().unwrap().starts_with(['{', '[']) {
                    true => self.value.clone().unwrap(),
                    false => format!("\"{}\"", self.value.clone().unwrap()),
                };

                format!("yq e -i '{} = {}' {}", self.key, value, self.file)
            }
        };
        cli_exec_from_dir(cmd, chart_local_path)?;
        Ok(())
    }

    fn get_path(&self) -> String {
        "".to_string()
    }

    fn set_path(&mut self, _new_path: String) {}

}

impl PatchInterface for RegexpPatch {
    fn apply(&self, chart_local_path: String) -> Result<(), Box<dyn std::error::Error>> {
        for entry in read_dir(self.path.clone())? {
            let entry = entry?;
            let filetype = entry.file_type()?;
            if filetype.is_dir() {
                error!(
                    "reading dirs is not supported yet, skipping {:?}",
                    entry.path()
                );
            } else {
                info!("reading a patch file: {:?}", entry.path());
                let config_content = File::open(entry.path())?;
                for patch_des in serde_yaml::Deserializer::from_reader(config_content) {
                    let patch: crate::patch::regexp::RegexpPatch =
                        match crate::patch::regexp::RegexpPatch::deserialize(patch_des) {
                            Ok(patch) => patch,
                            Err(err) => return Err(Box::from(err)),
                        };
                    info!("applying patch: {}", patch.name);
                    let after = match patch.after {
                        Some(after) => after,
                        None => "".to_string(),
                    };
                    match patch.before {
                        Some(before) => {
                            let patch_regexp = regex::Regex::new(before.as_str())?;
                            for target in patch.targets {
                                let file_path = format!("{}/{}", chart_local_path, target);
                                let file_content = fs::read_to_string(file_path.clone())?;
                                let new_content =
                                    patch_regexp.replace_all(file_content.as_str(), after.clone());
                                let mut file = OpenOptions::new()
                                    .write(true)
                                    .truncate(true)
                                    .open(file_path.clone())?;
                                file.write(new_content.as_bytes())?;
                            }
                        }
                        None => {
                            for target in patch.targets {
                                let file_path = format!("{}/{}", chart_local_path, target);
                                let file_content = fs::read_to_string(file_path.clone())?;
                                let new_content = format!("{}\n{}", file_content, after);
                                let mut file = OpenOptions::new()
                                    .write(true)
                                    .append(false)
                                    .open(file_path.clone())?;
                                file.write(new_content.as_bytes())?;
                            }
                        }
                    };
                }
            }
        }
        Ok(())
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn set_path(&mut self, new_path: String) {
        self.path = new_path
    }
}

impl PatchInterface for GitPatch {
    fn apply(&self, chart_local_path: String) -> Result<(), Box<dyn std::error::Error>> {
        if !is_git_repo(chart_local_path.clone()) {
            init_git_repo(chart_local_path.clone())?;
        };
        let cmd = format!("git -C {} apply {}", chart_local_path, self.path);
        cli_exec(cmd)?;
        remove_dir_all(chart_local_path + "/.git")?;
        Ok(())
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn set_path(&mut self, new_path: String) {
        self.path = new_path
    }
}

impl PatchInterface for CustomCommandPatch {
    fn apply(&self, chart_local_path: String) -> Result<(), Box<dyn std::error::Error>> {
        for cmd in self.commands.clone() {
            cli_exec_from_dir(cmd, chart_local_path.clone())?;
        }
        Ok(())
    }

    fn get_path(&self) -> String {
        // Empty stings, cause cc patch doesn't have a path
        "".to_string()
    }

    fn set_path(&mut self, _new_path: String) {
        ()
    }
}

fn patch_action_from_definition(
    patch: Patch,
) -> Result<Box<dyn PatchInterface>, Box<dyn std::error::Error>> {
    if let Some(regexp) = patch.regexp {
        Ok(Box::new(RegexpPatch { path: regexp.path }))
    } else if let Some(git) = patch.git {
        return Ok(Box::new(GitPatch {
            path: {
                let path = PathBuf::from(git.path.clone());
                match fs::canonicalize(path).ok() {
                    Some(can_path) => can_path.into_os_string().into_string().ok().unwrap(),
                    None => git.path.clone(),
                }
            },
        }));
    } else if let Some(custom_command) = patch.custom_command {
        return Ok(Box::new(CustomCommandPatch {
            commands: custom_command.commands,
        }));
    } else if let Some(yq) = patch.yq {
        if yq.op != YqOperations::Delete && yq.value.is_none() {
            return Err(Box::from("yq patch of non kind 'delete' requires a value"));
        };
        return Ok(Box::from(yq));
    } else {
        return Err(Box::from("unknown patch type"));
    }
}

fn is_git_repo(path: String) -> bool {
    let dot_git_path = path + ".git";
    Path::new(dot_git_path.as_str()).exists()
}

pub(crate) fn init_git_repo(path: String) -> Result<(), Box<dyn std::error::Error>> {
    cli_exec(format!("git -C {} init .", path))?;
    cli_exec(format!("git -C {} add .", path))?;
    cli_exec(format!("git -C {} commit -m 'Init commit'", path))?;
    Ok(())
}
