use crate::{config::Mirror, source::ChartLocal};

pub(crate) mod custom_command;
pub(crate) mod git;

pub(crate) trait Target {
    fn push(
        &self,
        workdir_path: String,
        chart_local: ChartLocal,
        dry_run: bool,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub(crate) fn mirror_from_mirror_obj(
    mirror: Mirror,
) -> Result<Box<dyn Target>, Box<dyn std::error::Error>> {
    if let Some(git) = mirror.git {
        return Ok(Box::from(git::Git {
            git_dir: match git.git_dir {
                Some(dir) => dir,
                None => mirror.name,
            },
            url: git.url,
            path: match git.path {
                Some(path) => path,
                None => "".to_string(),
            },
            branch: git.branch,
            commit: git.commit,
            default_branch: git.default_branch,
            rebase: match git.rebase {
                Some(r) => r,
                None => false,
            },
        }));
    } else if let Some(command) = mirror.custom_command {
        return Ok(Box::from(custom_command::CustomCommands {
            package: command.package,
            upload: command.upload,
        }));
    }
    Err(Box::from(format!(
        "a kind is unknown for the mirror {}",
        mirror.name
    )))
}
