use std::fs::{File, OpenOptions};
use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct CommandHistory {
    write_updates: bool,
    shell_history_file_path: PathBuf,
    command_history: VecDeque<String>,
}

impl CommandHistory {
  pub fn init (shell: &str, write_updates: bool) -> io::Result<Self>
  {
    let mut enable_write_updates = write_updates;

    // Determine the typical history file path based on the shell
    let history_file_path = match shell {
      "/bin/bash" => home::home_dir().unwrap().join(".bash_history"),
      "/bin/ksh" => home::home_dir().unwrap().join(".sh_history"),
      "/bin/tcsh" => home::home_dir().unwrap().join(".history"),
      "/bin/zsh" => home::home_dir().unwrap().join(".zsh_history"),
      _ => panic!("Unsupported shell"),
    };

    // Initialize the command_history vector
    let mut command_history = VecDeque::new();
    let shell_history_file = File::open(&history_file_path);

    match shell_history_file {
      Ok(file) => {
        let reader = BufReader::new(file);
        for line in reader.lines() {
          let line = line?;
          command_history.push_front(line);
        }
      },
      _ => {
        enable_write_updates = false;
      }
    }

    Ok(CommandHistory {
      write_updates: enable_write_updates,
      shell_history_file_path: history_file_path,
      command_history: command_history,
    })
  }

  pub fn get_history (&self) -> Vec<String> {
    Vec::from(self.command_history.clone())
  }

  pub fn maybe_append_command(&mut self, cmd: &str) -> io::Result<()> 
  {
    if self.write_updates {
      // Open the file in append mode
      let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&self.shell_history_file_path)?;

      // Append the command followed by a newline to the file
      writeln!(file, "{}", cmd)?;

      // Update the command_history vector as well
      self.command_history.push_front(cmd.to_string());
    }

    Ok(())
  }
}
