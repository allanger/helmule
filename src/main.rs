pub(crate) mod config;
pub(crate) mod helpers;
pub(crate) mod mirror;
pub(crate) mod patch;
pub(crate) mod source;

use clap::Parser;
use log::{error, info};
use std::collections::HashMap;
use std::{env, fs};
use std::{fs::create_dir, path::PathBuf, process::exit};
use tempfile::TempDir;

use crate::config::patch::init_git_repo;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the working dir
    #[arg(short, long)]
    workdir: Option<String>,
    /// Path to the configuration file
    #[arg(short, long)]
    config: String,
    /// Dry run
    #[arg(short, long, default_value = "false")]
    dry_run: bool,
    #[arg(long, default_value = "false")]
    skip_prerequisites_check: bool,
    /// Init git patch. Use it if you want to create git patch for a chart
    /// It's going to pull a chart and init a git repo there, so you can
    /// apply changes and create a patch file
    /// It's not going to try mirroring changes, but will apply extensions
    /// and patches that are already defined
    #[arg(long)]
    init_git_patch: Option<Vec<String>>,
}

fn check_prerequisites() -> Result<(), Box<dyn std::error::Error>> {
    info!("checking prerequisites");
    let prerequisites = vec!["helm", "yq", "helm"];
    for bin in prerequisites {
        info!("checking {}", bin);
        which::which(bin)?;
    }
    Ok(())
}

fn main() {
    // Prepare the logger
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();

    let args = Args::parse();
    if !args.skip_prerequisites_check {
        if let Err(err) = check_prerequisites() {
            error!("{}", err);
            exit(1);
        }
    }
    // Prepare the workdir
    let workdir_path = match args.workdir {
        Some(res) => match create_dir(res.clone()) {
            Ok(_) => {
                let path = PathBuf::from(res);
                let can_path = fs::canonicalize(path).ok().unwrap();
                can_path.into_os_string().into_string().ok().unwrap()
            }
            Err(err) => {
                error!("{}", err);
                exit(1);
            }
        },
        None => {
            let tmp_dir = match TempDir::new() {
                Ok(res) => res,
                Err(err) => {
                    error!("{}", err);
                    exit(1);
                }
            };
            match tmp_dir.path().to_str() {
                Some(res) => res.to_string(),
                None => {
                    exit(1);
                }
            }
        }
    };

    // Read the config
    let config = match config::Config::new(args.config) {
        Ok(res) => res,
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    };
    if let Some(charts) = config.clone().charts {
        for mut chart in charts {
            chart.populate_variables(config.variables.clone());
            match config.repositories.clone() {
                Some(repositories) => match chart.populate_repository(repositories) {
                    Ok(_) => {
                        info!("repo is populated for chart {}", chart.name);
                    }
                    Err(err) => {
                        error!("{}", err);
                        exit(1);
                    }
                },
                None => {
                    error!("repositories are not defined in the config");
                    exit(1);
                }
            };
            match config.mirrors.clone() {
                Some(mirrors) => match chart.populate_mirrors(mirrors) {
                    Ok(_) => {
                        info!("mirrors arepopulated for chart {}", chart.name)
                    }
                    Err(err) => {
                        error!("{}", err);
                        exit(1);
                    }
                },
                None => {
                    error!("mirrors are not defined in the config");
                    exit(1);
                }
            };
            let chart_repo = match source::repo_from_chart(chart.clone()) {
                Ok(res) => res,
                Err(err) => {
                    error!("{}", err);
                    exit(1);
                }
            };
            let vars = match chart.variables {
                Some(vars) => vars,
                None => HashMap::new(),
            };
            match chart_repo.pull(workdir_path.clone(), vars) {
                Ok(res) => {
                    info!(
                        "succesfully pulled chart {} into {}",
                        chart.name.clone(),
                        res.path,
                    );
                    if let Some(extensions) = chart.extensions.clone() {
                        for extension in extensions {
                            if let Err(err) = extension.apply(res.clone().path) {
                                error!("{}", err);
                                exit(1);
                            }
                        }
                    }
                    if let Some(patches) = chart.patches.clone() {
                        for patch in patches {
                            if let Err(err) = patch.apply(res.clone().path, config.patches.clone()) {
                                error!("{}", err);
                                exit(1);
                            }
                        }
                    }
                    if let Some(init_git_patch) = args.init_git_patch.clone() {
                        if init_git_patch.contains(&chart.name) {
                            info!(
                                "init git patch mode is enabled, go to {} to make your changes",
                                res.path
                            );
                            match init_git_repo(res.path) {
                                Ok(_) => {
                                    info!("not mirroring, because of the init git patch mode");
                                }
                                Err(err) => {
                                    error!("{}", err);
                                    exit(1);
                                }
                            };
                            break;
                        }
                    }
                    if let Some(mirrors) = chart.mirror_objs.clone() {
                        for mirror_obj in mirrors {
                            match mirror::mirror_from_mirror_obj(mirror_obj.clone()) {
                                Ok(mirror) => {
                                    match mirror.push(
                                        workdir_path.clone(),
                                        res.clone(),
                                        args.dry_run,
                                    ) {
                                        Ok(_) => info!(
                                            "mirrored {} to {}",
                                            chart.name.clone(),
                                            mirror_obj.name
                                        ),
                                        Err(err) => {
                                            error!("{}", err);
                                            exit(1);
                                        }
                                    };
                                }
                                Err(err) => {
                                    error!("{}", err);
                                    exit(1);
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("{}", err);
                    exit(1);
                }
            }
        }
    }

    // Populate charts
    // Download helm charts from config
    // If workdir is not provided, create a temporary di
}
