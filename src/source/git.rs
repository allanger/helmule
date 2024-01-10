use std::{fs::{self, rename}, collections::HashMap};

use crate::helpers::cli::{cli_exec, cli_exec_from_dir};
use base64::{engine::general_purpose, Engine as _};

use super::{ChartLocal, Repo};

pub(crate) struct Git {
    git_url: String,
    git_ref: String,
    path: String,
    pub(crate) chart: String,
}

impl From<crate::config::Chart> for Git {
    fn from(value: crate::config::Chart) -> Self {
        Git {
            git_url: value.get_git_repository_url(),
            git_ref: value.get_git_repository_ref(),
            path: value.get_git_repository_path(),
            chart: value.name,
        }
    }
}

impl Repo for Git {
    fn pull(&self, workdir_path: String, vars: HashMap<String, String>) -> Result<ChartLocal, Box<dyn std::error::Error>> {
        let repo_local_name = general_purpose::STANDARD_NO_PAD.encode(self.git_url.clone());
        let cmd = format!(
            "git clone {} {}/{}",
            self.git_url, workdir_path, repo_local_name
        );
        cli_exec(cmd)?;

        let cmd = format!(
            "git -C {}/{} checkout {}",
            workdir_path, repo_local_name, self.git_ref
        );
        cli_exec(cmd)?;

        let old_dir_name = format!(
            "{}/{}/{}/{}",
            workdir_path, repo_local_name, self.path, self.chart
        );
        let cmd = format!("helm show chart {}", old_dir_name);
        let helm_stdout = cli_exec(cmd)?;
        let new_dir_name: String;
        match serde_yaml::from_str::<super::Version>(&helm_stdout) {
            Ok(res) => {
                new_dir_name = format!("{}/{}-{}", workdir_path, self.chart, res.version);
                rename(old_dir_name, new_dir_name.clone())?;
            }
            Err(err) => return Err(Box::from(err)),
        };

        // Cleaning up
        fs::remove_dir_all(format!("{}/{}", workdir_path, repo_local_name))?;

        // Get the version
        let cmd = "helm show chart . | yq '.version'".to_string();
        let version = cli_exec_from_dir(cmd, new_dir_name.clone())?;
        Ok(ChartLocal {
            name: self.chart.clone(),
            version,
            path: new_dir_name,
            repo_url: self.git_url.clone(),
            vars,
        })
    }
}
