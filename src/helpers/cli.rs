use std::process::{Command, ExitStatus};

use log::info;

pub(crate) fn cli_exec(command: String) -> Result<String, Box<dyn std::error::Error>> {
    info!("executing: {}", command);
    let expect = format!("command has failed: {}", command);
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect(&expect);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !&output.status.success() {
        return Err(Box::from(stderr));
    };
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub(crate) fn cli_exec_from_dir(
    command: String,
    dir: String,
) -> Result<String, Box<dyn std::error::Error>> {
    info!("executing: {}", command);
    let expect = format!("command has failed: {}", command);
    let output = Command::new("sh")
        .arg("-c")
        .current_dir(dir)
        .arg(command)
        .output()
        .expect(&expect);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !&output.status.success() {
        return Err(Box::from(stderr));
    };
    let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
    stdout.pop();
    Ok(stdout)
}
#[cfg(test)]
mod tests {
    use crate::helpers::cli::{cli_exec, cli_exec_from_dir};

    #[test]
    fn test_stderr() {
        let command = ">&2 echo \"error\" && exit 1";
        let test = cli_exec(command.to_string());
        assert_eq!(test.err().unwrap().to_string(), "error\n".to_string());
    }

    #[test]
    fn test_stdout() {
        let command = "echo test";
        let test = cli_exec(command.to_string());
        assert_eq!(test.unwrap().to_string(), "test\n".to_string());
    }

    #[test]
    fn test_stdout_current_dir() {
        let dir = tempfile::tempdir().unwrap();
        let command = "echo $PWD";
        let dir_str = dir.into_path().into_os_string().into_string().unwrap();
        let test = cli_exec_from_dir(command.to_string(), dir_str.clone());
        assert!(test.unwrap().to_string().contains(dir_str.as_str()));
    }
}
