use crate::{helpers::cli::cli_exec_from_dir, source::ChartLocal, helpers::template};
use dircpy::*;

use super::Target;

pub(crate) struct Git {
    pub(crate) git_dir: String,
    pub(crate) url: String,
    pub(crate) path: String,
    pub(crate) branch: String,
    pub(crate) commit: Option<String>,
    pub(crate) default_branch: Option<String>,
    pub(crate) rebase: bool,
}

impl Target for Git {
    fn push(
        &self,
        workdir_path: String,
        chart_local: ChartLocal,
        dry_run: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Prepare the URL
        let url = template::render(self.url.clone(), &chart_local)?;
        //Prepare the git dir
        let git_dir = template::render(self.git_dir.clone(), &chart_local)?;
        let cmd = format!("git clone {} {}", url, git_dir);
        cli_exec_from_dir(cmd, workdir_path.clone())?;
        let git_repo_path = format!("{}/{}", workdir_path, git_dir);

        // Prepare branch
        let branch = template::render(self.branch.clone(), &chart_local)?;
        let cmd = format!("git checkout {}", branch);
        if let Err(_) = cli_exec_from_dir(cmd, git_repo_path.clone()) {
            let cmd = format!("git checkout -b {}", branch);
            cli_exec_from_dir(cmd, git_repo_path.clone())?;
        };
        let mut git_args: String = String::new();
        if self.rebase {
            let default_branch = match self.default_branch.clone() {
                Some(db) => db,
                None => "main".to_string(),
            };
            let cmd = format!("git rebase {}", default_branch);
            cli_exec_from_dir(cmd, git_repo_path.clone())?;
            git_args = "--force".to_string();
        }
        // Prepare path
        let path = template::render(self.path.clone(), &chart_local)?;
        let repo_local_full_path = format!("{}/{}", git_repo_path, path);
        CopyBuilder::new(chart_local.path.clone(), repo_local_full_path.clone())
            .overwrite_if_size_differs(true)
            .run()?;

        // Prepare the commit message
        let commit_message = match self.commit.clone() {
            Some(commit) => commit,
            None => "helmuled {{ name }}-{{ version }}".to_string(),
        };
        let commit = template::render(commit_message.clone(), &chart_local)?;
        let cmd = format!(
            "git add . && git diff --staged --quiet || git commit -m '{}'",
            commit
        );
        cli_exec_from_dir(cmd, repo_local_full_path.clone())?;
        if !dry_run {
            let cmd = format!("git push --set-upstream origin {} {}", branch, git_args);
            cli_exec_from_dir(cmd, repo_local_full_path)?;
        }
        Ok(())
    }
}
