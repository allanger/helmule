use std::collections::HashMap;

use crate::config::Chart;
use serde::{Deserialize, Serialize};

pub(crate) mod git;
pub(crate) mod helm;

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct ChartLocal {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) path: String,
    pub(crate) repo_url: String,
    pub(crate) vars: HashMap<String, String>,
}

impl ChartLocal {
    pub(crate) fn test(&self) -> String {
        "test-me-if-you-can".to_string()
    }
}

pub(crate) trait Repo {
    fn pull(
        &self,
        workdir_path: String,
        vars: HashMap<String, String>,
    ) -> Result<ChartLocal, Box<dyn std::error::Error>>;
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Version {
    pub(crate) version: String,
}

pub(crate) fn repo_from_chart(chart: Chart) -> Result<Box<dyn Repo>, Box<dyn std::error::Error>> {
    match chart.get_repo_kind() {
        Ok(res) => match res {
            crate::config::RepositoryKind::Helm => {
                let helm: helm::Helm = chart.into();
                Ok(Box::new(helm))
            }
            crate::config::RepositoryKind::Git => {
                let git: git::Git = chart.into();
                Ok(Box::new(git))
            }
        },
        Err(err) => Err(err),
    }
}
