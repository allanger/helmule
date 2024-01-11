use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};

pub(crate) mod extension;
pub(crate) mod patch;
pub(crate) mod include;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Config {
    pub(crate) include: Option<HashMap::<include::IncludeTypes, String>>,
    pub(crate) variables: Option<HashMap<String, String>>,
    pub(crate) repositories: Option<Vec<Repository>>,
    pub(crate) charts: Option<Vec<Chart>>,
    pub(crate) mirrors: Option<Vec<Mirror>>,
}

impl Config {
    pub(crate) fn new(config_path: String) -> Result<Self, Box<dyn std::error::Error>> {
        info!("reading the config file");
        let config_content = File::open(config_path)?;
        let mut config: Config = serde_yaml::from_reader(config_content)?;
        if config.include.is_some() {
            include::apply_includes(&mut config)?;
        }
        Ok(config)
    }
}

pub(crate) enum RepositoryKind {
    Helm,
    Git,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Repository {
    // A name of the repository to be references by charts
    pub(crate) name: String,
    // Helm repository data
    pub(crate) helm: Option<Helm>,
    pub(crate) git: Option<Git>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Mirror {
    pub(crate) name: String,
    pub(crate) git: Option<GitMirror>,
    pub(crate) custom_command: Option<CustomCommandsMirror>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct CustomCommandsMirror {
    pub(crate) package: Vec<String>,
    pub(crate) upload: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct GitMirror {
    pub(crate) url: String,
    pub(crate) path: Option<String>,
    pub(crate) branch: String,
    pub(crate) commit: Option<String>,
    pub(crate) git_dir: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Helm {
    // A url of the helm repository
    pub(crate) url: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Git {
    pub(crate) url: String,
    #[serde(alias = "ref")]
    pub(crate) git_ref: String,
    pub(crate) path: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Chart {
    // A name of the helm chart
    pub(crate) name: String,
    // A reference to repository by name
    pub(crate) repository: String,
    pub(crate) mirrors: Vec<String>,
    // Versions to be mirrored
    pub(crate) version: Option<String>,
    // A repository object
    pub(crate) extensions: Option<Vec<extension::Extension>>,
    pub(crate) patches: Option<Vec<patch::Patch>>,
    pub(crate) variables: Option<HashMap<String, String>>,
    #[serde(skip_serializing)]
    pub(crate) repository_obj: Option<Repository>,
    #[serde(skip_serializing)]
    pub(crate) mirror_objs: Option<Vec<Mirror>>,
}

impl Chart {
    pub(crate) fn populate_variables(&mut self, global_variables: Option<HashMap<String, String>>) {
        if let Some(global_vars) = global_variables {
            self.variables = match self.variables.clone() {
                Some(mut vars) => {
                    vars.extend(global_vars);
                    Some(vars)
                }
                None => Some(global_vars),
            }
        };
    }

    pub(crate) fn populate_repository(
        &mut self,
        repositories: Vec<Repository>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for repository in repositories {
            if repository.name == self.repository {
                self.repository_obj = Some(repository);
                return Ok(());
            }
        }
        //let err = error!("repo {} is not found in the repo list", self.repository);
        let error_msg = format!("repo {} is not found in the repo list", self.repository);
        Err(Box::from(error_msg))
    }
    // TODO: Handle the "mirror not found" error
    pub(crate) fn populate_mirrors(
        &mut self,
        mirrors: Vec<Mirror>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut mirror_objs: Vec<Mirror> = vec![];

        for mirror_global in mirrors.clone() {
            for mirror_name in self.mirrors.clone() {
                if mirror_name == mirror_global.name.clone() {
                    mirror_objs.push(mirror_global.clone());
                }
            }
        }
        if !mirror_objs.is_empty() {
            self.mirror_objs = Some(mirror_objs);
        }
        Ok(())
    }

    pub(crate) fn get_helm_repository_url(&self) -> String {
        match self.repository_obj.clone() {
            Some(res) => res.helm.unwrap().url,
            None => {
                warn!("repository object is not filled for chart {}", self.name);
                "".to_string()
            }
        }
    }

    pub(crate) fn get_git_repository_url(&self) -> String {
        match self.repository_obj.clone() {
            Some(res) => res.git.unwrap().url,
            None => {
                warn!("repository object is not filled for chart {}", self.name);
                "".to_string()
            }
        }
    }

    pub(crate) fn get_git_repository_ref(&self) -> String {
        match self.repository_obj.clone() {
            Some(res) => res.git.unwrap().git_ref,
            None => {
                warn!("repository object is not filled for chart {}", self.name);
                "".to_string()
            }
        }
    }

    pub(crate) fn get_git_repository_path(&self) -> String {
        match self.repository_obj.clone() {
            Some(res) => res.git.unwrap().path,
            None => {
                warn!("repository object is not filled for chart {}", self.name);
                "".to_string()
            }
        }
    }

    pub(crate) fn get_repo_kind(&self) -> Result<RepositoryKind, Box<dyn std::error::Error>> {
        match &self.repository_obj {
            Some(res) => {
                if res.helm.is_some() {
                    Ok(RepositoryKind::Helm)
                } else if res.git.is_some() {
                    return Ok(RepositoryKind::Git);
                } else {
                    return Err(Box::from("unknown repository kind is found"));
                }
            }
            None => Err(Box::from(
                "repository object is not filled up for the chart",
            )),
        }
    }
}
