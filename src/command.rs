use std::io::Error;
use std::{io, process};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[derive(Clone)]
pub struct CommandOutput {
  pub success: bool,
  pub status_code: i32,
  pub stdout: String,
  pub stderr: String
}

/// Captures the output of a comamnd executed within the underlying shell. This includes the status code, the
/// standard output, and standard error output. `CommandOutput` resembles `process::Output` but also handles 
/// byte-vector to UTF8 string conversion when accessing outputs.
impl CommandOutput {
  fn from (output: process::Output) -> io::Result<CommandOutput>
  {
    let stdout = std::str::from_utf8(&output.stdout);
    let stderr = std::str::from_utf8(&output.stderr);

    match (stdout, stderr) {
      (Ok(stdout), Ok(stderr)) => {
        Ok(
          CommandOutput {
            success: output.status.success(),
            status_code: output.status.code().unwrap_or(-1),
            stdout: stdout.to_string(),
            stderr: stderr.to_string()
          }
        )
     },
      (Err(e), _) => Err(io::Error::new(io::ErrorKind::Other, e)),
      (_, Err(e)) => Err(io::Error::new(io::ErrorKind::Other, e))
    }
  }

  /// Useful for testing.
  #[cfg(test)]
  fn from_fields (success: bool, status_code: i32, stdout: String, stderr: String) -> CommandOutput
  {
    CommandOutput {
      success: success,
      status_code: status_code,
      stdout: stdout,
      stderr: stderr
    }
  }
}

/// A `CommandExecutor` is responsible for interfacing with underlying system commands; either checking for the existence
/// of a proposed command by a model, or executing a command and returning its output to the caller.
#[cfg_attr(test, automock)]
pub trait CommandExecutorInterface {
  fn exists(&self, shell: &str, command: &str) -> bool;
  fn execute(&self, shell: &str, command: &str) -> Result<CommandOutput, Error>;
}

pub struct CommandExecutor;

impl CommandExecutorInterface for CommandExecutor {
  /// Used for checking for existence of a command, by passing it to 
  ///   $ `$SHELL -c command -v <command string>`
  /// 
  /// Which will return success (and a path to the command) if it exists; otherwise, or upon 
  /// error, returns false.
  fn 
  exists(&self, shell: &str, command: &str) -> bool
  {
    match std::process::Command::new(shell)
      .arg("-c")
      .arg(format!("command -v {}", command))
      .output()
      .map(|output| CommandOutput::from(output)) {
        Ok(output) => {
          match output {
            Ok(output) => output.success,
            Err(_) => false
          }
        },
        Err(_) => false
      }
  }

  /// Executes a command, by passing it to 
  ///   $ `$SHELL -c <command string>`
  /// 
  /// Returns the collected status code, stdout and stderr wrapped in a `CommandOutput` object
  fn 
  execute(&self, shell: &str, command: &str) -> Result<CommandOutput, Error> 
  {
    std::process::Command::new(shell)
      .arg("-c")
      .arg(command)
      .output()
      .map(|output| CommandOutput::from(output))?
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_command_exists() {
    let mut mock_executor = MockCommandExecutorInterface::new();
    mock_executor.expect_exists()
      .with(eq("/bin/bash"), eq("echo"))
      .returning(|_, _| true);
    
    assert!(mock_executor.exists("/bin/bash", "echo"));
  }

  #[test]
  fn test_command_execute() {
    let mut mock_executor = MockCommandExecutorInterface::new();
    let command_output = CommandOutput::from_fields(true, 0, "Hello".to_string(), "".to_string());
    mock_executor.expect_execute()
      .with(eq("/bin/bash"), eq("echo Hello"))
      .returning(move |_, _| Ok(command_output.clone()));

    match mock_executor.execute("/bin/bash", "echo Hello") {
      Ok(output) => assert_eq!(output.stdout, "Hello"),
      Err(_) => panic!("Execution should succeed"),
  }
  }
}
