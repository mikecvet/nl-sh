use regex::Regex;
use std::env;
use std::io::{self, Error};

pub use crate::command_history::*;

/// Context about the environment in which this shell is being run. 
pub struct Context {
  pub uname: String,
  pub shell: String,
  pub os: String,
  pub pwd: String,
  pub history: CommandHistory
}

/// Determine the environment's current working directory.
pub fn
get_current_working_dir () -> Result<String, Error> 
{
  let current_dir = env::current_dir()?;
  current_dir.into_os_string().into_string().map_err(|_| {
    std::io::Error::new(std::io::ErrorKind::InvalidData, "Current directory contains invalid Unicode data")
  })
}

/// Sanitizes stdout read from executed `Command`s. 
pub fn 
sanitize_stdout (stdout: &str) -> String
{
  // Current santization is eliminating unecessary whitespace
  let re = Regex::new(r"\s+").unwrap();
  re.replace_all(stdout.trim(), " ").into_owned()
}

impl Context 
{
  pub fn 
  init (uname: Vec<u8>, shell: &str, os: Vec<u8>) -> io::Result<Context> 
  {
    Ok(Context {
      uname: sanitize_stdout(std::str::from_utf8(&uname).expect("failed to convert stdout to String")),
      shell: shell.to_string(),
      os: sanitize_stdout(std::str::from_utf8(&os).expect("failed to convert stdout to String")),
      pwd: get_current_working_dir().unwrap(),
      history: CommandHistory::init(shell, true)?
    })
  }

  pub fn 
  update_command (&mut self, cmd: &str) 
  {
    self.history.maybe_append_command(cmd);
  }

  pub fn 
  get_command_history (&self) -> Vec<String>
  {
    self.history.get_history()
  }
}
