use super::{ChartLocal, Repo};
use crate::helpers::cli::{cli_exec, cli_exec_from_dir};
use base64::{engine::general_purpose, Engine as _};
use std::{collections::HashMap, fs::rename};

pub(crate) enum RepoKind {
    Default,
    Oci,
}

const LATEST_VERSION: &str = "latest";

pub(crate) struct Helm {
    pub(crate) chart: String,
    pub(crate) repository_url: String,
    pub(crate) version: String,
}

impl From<crate::config::Chart> for Helm {
    fn from(value: crate::config::Chart) -> Self {
        Helm {
            chart: value.name.clone(),
            repository_url: value.get_helm_repository_url(),
            version: match value.version {
                Some(res) => res,
                None => LATEST_VERSION.to_string(),
            },
        }
    }
}

impl Helm {
    fn repo_kind_from_url(&self) -> Result<RepoKind, Box<dyn std::error::Error>> {
        let prefix = self
            .repository_url
            .chars()
            .take_while(|&ch| ch != ':')
            .collect::<String>();
        match prefix.as_str() {
            "oci" => Ok(RepoKind::Oci),
            "https" | "http" => Ok(RepoKind::Default),
            _ => Err(Box::from(format!(
                "repo kind is not defined by the prefix: {}",
                prefix
            ))),
        }
    }

    fn pull_default(
        &self,
        workdir_path: String,
        vars: HashMap<String, String>,
    ) -> Result<ChartLocal, Box<dyn std::error::Error>> {
        // Add repo and update
        let repo_local_name = general_purpose::STANDARD_NO_PAD.encode(self.repository_url.clone());
        let cmd = format!("helm repo add {} {}", repo_local_name, self.repository_url);
        cli_exec(cmd)?;
        cli_exec("helm repo update".to_string())?;

        let args = match self.version.as_str() {
            LATEST_VERSION => "".to_string(),
            _ => format!("--version {}", self.version.clone()),
        };
        let cmd = format!(
            "helm pull {}/{} {} --destination {} --untar",
            repo_local_name, &self.chart, args, workdir_path
        );
        cli_exec(cmd)?;

        // Get the version
        let cmd = format!("helm show chart {}/{}", workdir_path, &self.chart);
        let helm_stdout = cli_exec(cmd)?;
        let old_dir_name = format!("{}/{}", workdir_path, &self.chart);
        let new_dir_name: String;
        match serde_yaml::from_str::<super::Version>(&helm_stdout) {
            Ok(res) => {
                new_dir_name = format!("{}-{}", old_dir_name, res.version);
                rename(old_dir_name, new_dir_name.clone())?;
            }
            Err(err) => return Err(Box::from(err)),
        };

        //cleaning up
        let cmd = format!("helm repo remove {}", repo_local_name);
        cli_exec(cmd)?;

        let cmd = "helm show chart . | yq '.version'".to_string();
        let version = cli_exec_from_dir(cmd, new_dir_name.clone())?;
        Ok(ChartLocal {
            name: self.chart.clone(),
            version,
            path: new_dir_name,
            repo_url: self.repository_url.clone(),
            vars,
        })
    }
}

impl Repo for Helm {
    fn pull(
        &self,
        workdir_path: String,
        vars: HashMap<String, String>,
    ) -> Result<ChartLocal, Box<dyn std::error::Error>> {
        let repository_kind = self.repo_kind_from_url()?;
        let path = match repository_kind {
            RepoKind::Default => self.pull_default(workdir_path, vars)?,
            RepoKind::Oci => {
                todo!()
            }
        };
        Ok(path)
    }
}
