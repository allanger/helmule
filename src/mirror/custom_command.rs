use super::Target;
use crate::{helpers::cli::cli_exec_from_dir, helpers::template};

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
            let mut reg = template::register_handlebars();
            reg.register_template_string("cmd", cmd_tmpl)?;
            let cmd = reg.render("cmd", &chart_local)?;
            cli_exec_from_dir(cmd, workdir_path.clone())?;
        }
        if !dry_run {
            for cmd_tmpl in self.upload.clone() {
                let mut reg = template::register_handlebars();
                reg.register_template_string("cmd", cmd_tmpl)?;
                let cmd = reg.render("cmd", &chart_local)?;
                cli_exec_from_dir(cmd, workdir_path.clone())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CustomCommands;
    use crate::{mirror::Target, source::ChartLocal};
    use std::{collections::HashMap, fs::create_dir_all, path::Path};
    use tempfile::TempDir;

    fn get_chart_local() -> crate::source::ChartLocal {
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("key".to_string(), "value".to_string());
        ChartLocal {
            name: "chart".to_string(),
            version: "1.0.0".to_string(),
            path: "chart-1.0.0".to_string(),
            repo_url: "https:://helm.repo".to_string(),
            vars,
        }
    }

    fn prepare_test_workdir(chart_path: String) -> String {
        let workdir = TempDir::new().unwrap().path().to_str().unwrap().to_string();
        println!("test workdir is {}", workdir.clone());
        create_dir_all(format!("{}/{}", workdir, chart_path)).unwrap();
        workdir
    }

    #[test]
    fn test_package_basic() {
        let chart_local = get_chart_local();
        let workdir = prepare_test_workdir(chart_local.path.clone());

        let custom_commands = CustomCommands {
            package: vec!["touch package".to_string()],
            upload: vec!["touch upload".to_string()],
        };

        let cc_target: Box<dyn Target> = Box::from(custom_commands);
        cc_target.push(workdir.clone(), chart_local, true).unwrap();

        assert!(Path::new(&format!("{}/package", workdir)).exists());
        assert!(!Path::new(&format!("{}/upload", workdir)).exists());
    }

    #[test]
    fn test_upload_basic() {
        let chart_local = get_chart_local();
        let workdir = prepare_test_workdir(chart_local.path.clone());

        let custom_commands = CustomCommands {
            package: vec!["touch package".to_string()],
            upload: vec!["touch upload".to_string()],
        };

        let cc_target: Box<dyn Target> = Box::from(custom_commands);
        cc_target.push(workdir.clone(), chart_local, false).unwrap();

        assert!(Path::new(&format!("{}/package", workdir)).exists());
        assert!(Path::new(&format!("{}/upload", workdir)).exists());
    }

    #[test]
    fn test_templates() {
        let chart_local = get_chart_local();
        let workdir = prepare_test_workdir(chart_local.path.clone());

        let custom_commands = CustomCommands {
            package: vec!["touch {{ name }}-{{ version }}".to_string()],
            upload: vec!["touch {{ repo_url }}-{{ vars.key }}".to_string()],
        };

        let cc_target: Box<dyn Target> = Box::from(custom_commands);
        cc_target
            .push(workdir.clone(), chart_local.clone(), true)
            .unwrap();

        assert!(Path::new(&format!(
            "{}/{}-{}",
            workdir, chart_local.name, chart_local.version
        ))
        .exists());
        assert!(!Path::new(&format!(
            "{}/{}-{}",
            workdir,
            chart_local.repo_url,
            chart_local.vars.get("key").unwrap()
        ))
        .exists());
    }
}
