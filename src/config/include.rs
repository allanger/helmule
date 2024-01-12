use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, Hash, PartialEq, Debug, Clone)]
pub(crate) enum IncludeTypes {
    Charts,
    Mirrors,
    Repositories,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Include {
    kind: IncludeTypes,
    path: String,
}

pub(crate) fn apply_includes(config: &mut super::Config) -> Result<(), Box<dyn std::error::Error>> {
    for include in config.include.clone().unwrap() {
        match include.kind {
            IncludeTypes::Charts => {
                let mut charts = include_chart(include.path)?;
                let extended_charts = match config.charts.clone() {
                    Some(mut existing_charts) => {
                        existing_charts.append(&mut charts);
                        existing_charts
                    }
                    None => charts,
                };
                config.charts = Some(extended_charts);
            }
            IncludeTypes::Mirrors => {
                let mut mirrors = include_mirrors(include.path)?;
                let extended_mirrors = match config.mirrors.clone() {
                    Some(mut existing_mirrors) => {
                        existing_mirrors.append(&mut mirrors);
                        existing_mirrors
                    }
                    None => mirrors,
                };
                config.mirrors = Some(extended_mirrors);
            }
            IncludeTypes::Repositories => {
                let mut repositories = include_repositories(include.path)?;
                let extended_repositories = match config.repositories.clone() {
                    Some(mut existing_repositories) => {
                        existing_repositories.append(&mut repositories);
                        existing_repositories
                    }
                    None => repositories,
                };
                config.repositories = Some(extended_repositories);
            }
        };
    }
    Ok(())
}

fn include_chart(path: String) -> Result<Vec<super::Chart>, Box<dyn std::error::Error>> {
    info!("trying to include chart from {}", path.clone());
    let file = std::fs::File::open(path.clone())?;

    let chart_dir = match std::path::Path::new(&path).parent() {
        Some(dir) => match dir.to_str() {
            Some(dir) => dir.to_string(),
            None => {
                return Err(Box::from(format!(
                    "chart parrent dir not found for {}",
                    path
                )));
            }
        },
        None => {
            return Err(Box::from(format!(
                "chart parrent dir not found for {}",
                path
            )));
        }
    };

    let mut charts: Vec<super::Chart> = match serde_yaml::from_reader(file) {
        Ok(res) => res,
        Err(_) => {
            let file = std::fs::File::open(path.clone())?;
            let chart: super::Chart = serde_yaml::from_reader(file)?;
            vec![chart]
        }
    };

    charts.iter_mut().for_each(|chart| {
        match chart.extensions {
            Some(ref mut extensions) => extensions.iter_mut().for_each(|extension| {
                if is_path_relative(extension.source_dir.clone()) {
                    let clean_path = match extension.source_dir.clone().starts_with("./") {
                        true => extension.source_dir.clone().replacen("./", "", 1),
                        false => extension.source_dir.clone(),
                    };
                    if is_path_relative(clean_path.clone()) {
                        let new_path = format!("{}/{}", chart_dir, clean_path);
                        extension.source_dir = new_path;
                    }
                }
            }),
            None => info!("no extensions set, nothing to update"),
        };
        match chart.patches {
            Some(ref mut patches) => patches.iter_mut().for_each(| patch| {
                if is_path_relative(patch.get_path().clone()) {
                    let clean_path = match patch.get_path().clone().starts_with("./") {
                        true => patch.get_path().clone().replacen("./", "", 1),
                        false => patch.get_path().clone(),
                    };
                    if is_path_relative(clean_path.clone()) {
                        let new_path = format!("{}/{}", chart_dir, clean_path);
                        patch.set_path(new_path);
                    }
                }
            }),
            None => info!("no patch set, nothing to update"),
        };

    });
    Ok(charts)
}

fn is_path_relative(path: String) -> bool {
    !path.starts_with("/")
}

fn include_mirrors(path: String) -> Result<Vec<super::Mirror>, Box<dyn std::error::Error>> {
    info!("trying to include chart from {}", path.clone());
    let file = std::fs::File::open(path.clone())?;
    let mirrors: Vec<super::Mirror> = match serde_yaml::from_reader(file) {
        Ok(res) => res,
        Err(_) => {
            let file = std::fs::File::open(path.clone())?;
            let chart: super::Mirror = serde_yaml::from_reader(file)?;
            vec![chart]
        }
    };
    Ok(mirrors)
}

fn include_repositories(
    path: String,
) -> Result<Vec<super::Repository>, Box<dyn std::error::Error>> {
    info!("trying to include chart from {}", path.clone());
    let file = std::fs::File::open(path.clone())?;
    let repositories: Vec<super::Repository> = match serde_yaml::from_reader(file) {
        Ok(res) => res,
        Err(_) => {
            let file = std::fs::File::open(path.clone())?;
            let chart: super::Repository = serde_yaml::from_reader(file)?;
            vec![chart]
        }
    };
    Ok(repositories)
}
