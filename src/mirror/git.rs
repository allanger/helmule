use crate::{helpers::cli::cli_exec_from_dir, source::ChartLocal};
use dircpy::*;

use super::Target;

pub(crate) struct Git {
    pub(crate) git_dir: String,
    pub(crate) url: String,
    pub(crate) path: String,
    pub(crate) branch: String,
    pub(crate) commit: Option<String>,
}

impl Target for Git {
    fn push(
        &self,
        workdir_path: String,
        chart_local: ChartLocal,
        dry_run: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut reg = super::register_handlebars();
        // Prepare the URL
        reg.register_template_string("url", self.url.clone())?;
        let url = reg.render("url", &chart_local)?;
        //Prepare the git dir
        reg.register_template_string("git_dir", self.git_dir.clone())?;
        let git_dir = reg.render("git_dir", &chart_local)?;

        let cmd = format!("git clone {} {}", url, git_dir);
        cli_exec_from_dir(cmd, workdir_path.clone())?;
        let git_repo_path = format!("{}/{}", workdir_path, git_dir);

        // Prepare branch
        reg.register_template_string("branch", self.branch.clone())?;
        let branch = reg.render("branch", &chart_local)?;
        let cmd = format!("git checkout {}", branch);
        if let Err(_) = cli_exec_from_dir(cmd, git_repo_path.clone()) {
            let cmd = format!("git checkout -b {}", branch);
            cli_exec_from_dir(cmd, git_repo_path.clone())?;
        };
        // Prepare path
        reg.register_template_string("path", self.path.clone())?;
        let path = reg.render("path", &chart_local)?;
        let repo_local_full_path = format!("{}/{}", git_repo_path, path);
        CopyBuilder::new(chart_local.path.clone(), repo_local_full_path.clone())
            .overwrite_if_size_differs(true)
            .run()?;

        // Prepare the commit message
        let commit_message = match self.commit.clone() {
            Some(commit) => commit,
            None => "helmuled {{ name }}-{{ version }}".to_string(),
        };
        reg.register_template_string("commit", commit_message.clone())?;
        let commit = reg.render("commit", &chart_local)?;
        let cmd = format!(
            "git add . && git diff --staged --quiet || git commit -m '{}'",
            commit
        );
        cli_exec_from_dir(cmd, repo_local_full_path.clone())?;
        if !dry_run {
            let cmd = format!("git push --set-upstream origin {}", branch);
            cli_exec_from_dir(cmd, repo_local_full_path)?;
        }
        Ok(())
    }
}
