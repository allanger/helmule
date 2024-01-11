use log::info;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, Hash, PartialEq, Debug, Clone)]
pub(crate) enum IncludeTypes {
    Charts,
    Mirrors,
    Repositories,
}

pub(crate) fn apply_includes(config: &mut super::Config) -> Result<(), Box<dyn std::error::Error>> {
    for (kind, path) in config.include.clone().unwrap() {
        match kind {
            IncludeTypes::Charts => {
                let mut charts = include_chart(path)?;
                let extended_charts = match config.charts.clone() {
                    Some(mut existing_charts) => {
                        existing_charts.append(&mut charts);
                        existing_charts
                    },
                    None => charts,
                };
                config.charts = Some(extended_charts); 
            },
            IncludeTypes::Mirrors => {
                let mut mirrors = include_mirrors(path)?;
                let extended_mirrors = match config.mirrors.clone() {
                    Some(mut existing_mirrors) => {
                        existing_mirrors.append(&mut mirrors);
                        existing_mirrors
                    },
                    None => mirrors,
                };
                config.mirrors = Some(extended_mirrors);
            },
            IncludeTypes::Repositories => {
                let mut repositories = include_repositories(path)?;
                let extended_repositories = match config.repositories.clone() {
                    Some(mut existing_repositories) => {
                        existing_repositories.append(&mut repositories);
                        existing_repositories
                    },
                    None => repositories,
                };
                config.repositories = Some(extended_repositories);
            },
        };
    }
    Ok(())
}

fn include_chart(path: String) -> Result<Vec<super::Chart>, Box<dyn std::error::Error>> {
    info!("trying to include chart from {}", path.clone());
    let file = std::fs::File::open(path.clone())?;
    let charts: Vec<super::Chart> = match serde_yaml::from_reader(file) {
        Ok(res) => res,
        Err(_) => {
            let file = std::fs::File::open(path.clone())?;
            let chart: super::Chart = serde_yaml::from_reader(file)?;
            vec!(chart)
        },
    };
    Ok(charts)
}

fn include_mirrors(path: String) -> Result<Vec<super::Mirror>, Box<dyn std::error::Error>> {
    info!("trying to include chart from {}", path.clone());
    let file = std::fs::File::open(path.clone())?;
    let mirrors: Vec<super::Mirror> = match serde_yaml::from_reader(file) {
        Ok(res) => res,
        Err(_) => {
            let file = std::fs::File::open(path.clone())?;
            let chart: super::Mirror = serde_yaml::from_reader(file)?;
            vec!(chart)
        },
    };
    Ok(mirrors)
}

fn include_repositories(path: String) -> Result<Vec<super::Repository>, Box<dyn std::error::Error>> {
    info!("trying to include chart from {}", path.clone());
    let file = std::fs::File::open(path.clone())?;
    let repositories: Vec<super::Repository> = match serde_yaml::from_reader(file) {
        Ok(res) => res,
        Err(_) => {
            let file = std::fs::File::open(path.clone())?;
            let chart: super::Repository = serde_yaml::from_reader(file)?;
            vec!(chart)
        },
    };
    Ok(repositories)
}

