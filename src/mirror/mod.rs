use chrono::prelude::*;
use handlebars::{handlebars_helper, Handlebars};
use time::{format_description::parse, OffsetDateTime};

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
            git_dir: mirror.name.clone(),
            url: git.url,
            path: match git.path {
                Some(path) => path,
                None => "".to_string(),
            },
            branch: git.branch,
            commit: git.commit,
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

handlebars_helper!(date_helper: | | Utc::now().format("%Y-%m-%d").to_string());
handlebars_helper!(time_helper: | | Utc::now().format("%H-%M-%S").to_string());

pub(crate) fn register_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("date", Box::new(date_helper));
    handlebars.register_helper("time", Box::new(time_helper));
    handlebars
}
