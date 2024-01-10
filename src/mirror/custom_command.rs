use crate::helpers::cli::cli_exec_from_dir;

use super::Target;

pub(crate) struct CustomCommands {
    pub(crate) package: Vec<String>,
    pub(crate) upload: Vec<String>,
}

impl Target for CustomCommands {
    fn push(
        &self,
        workdir_path: String,
        chart_local: crate::source::ChartLocal,
        dry_run: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for cmd_tmpl in self.package.clone() {
            let mut reg = super::register_handlebars();
            reg.register_template_string("cmd", cmd_tmpl)?;
            let cmd = reg.render("cmd", &chart_local)?;
            cli_exec_from_dir(cmd, workdir_path.clone())?;
        }
        if !dry_run {
            for cmd_tmpl in self.upload.clone() {
                let mut reg = super::register_handlebars();
                reg.register_template_string("cmd", cmd_tmpl)?;
                let cmd = reg.render("cmd", &chart_local)?;
                cli_exec_from_dir(cmd, workdir_path.clone())?;
            }
        }
        Ok(())
    }
}
