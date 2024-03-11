use inquire::{Confirm, Text};
use inquire::error::InquireError;
use inquire::history::SimpleHistory;
use std::env;
use std::process::Command;

pub use crate::context::*;
pub use crate::model::*;

/// Some POSIX commands have verb-y characteristics, and for these, we'll let the LLM determine
/// whether the user intent is to run a specfic command, or whether the request is something
/// akin to "diff these two files" or "sort and print the text in a, b, and c"
static COMMAND_EXCEPTIONS: &[&str] = &["alias", "cat", "diff", "expand", "find", "kill", "link", 
  "list", "log", "read", "sort", "split", "strip", "touch", "type", "what", "which", "who"];

/// Determine whether the input from the prompt is a likely system command. This is
/// a best-effort attempt to short-circut requests to an LLM, to improve performance
/// for the most obvious native command interpreted by the prompt.
/// 
/// In the case where the user request is ambiguous, as defined by the set 
/// of COMMAND_EXCEPTIONS strings, fall through and indicate to pass the user input
/// to the LLM to rationalize.
fn 
likely_system_command (context: &Context, command: &String) -> bool {
  let parts: Vec<&str> = command.split_whitespace().collect();
  let cmd = parts.get(0).unwrap_or(&"").to_lowercase(); // Extract the just command without arguments

  if COMMAND_EXCEPTIONS.contains(&cmd.as_str()) {
    // The user string contains a verb-y command, let the LLM sort it out
    return false
  } else {
    // Determine whether the first word in the user input is valid command on this system, 
    // via $SHELL command -v <cmd>
    Command::new(context.shell.clone())
      .arg("-c")
      .arg(format!("command -v {}", cmd))
      .output()
      .map(|output| output.status.success())
      .unwrap_or(false)
  }
}

/// Conditionally updates the given `Context`, depending on the nature of the sucessfullly-executed command string.
fn 
maybe_update_context (cmd_input: &str, context: &mut Context) -> Result<(), Box<dyn std::error::Error>>
{
  let mut parts: Vec<&str> = cmd_input.split_whitespace().collect();
  let cmd = parts.remove(0);

  if cmd.to_owned().to_lowercase().eq("cd") && parts.len() > 1 {
    // If this was a change-directory command, set the current environment to
    // cd's subsequent argument and update `context.pwd`
    env::set_current_dir(parts.remove(0))?;
    context.pwd = get_current_working_dir()?;
  }

  // Possibly update command history with this most recent command
  context.update_command(cmd_input)?;

  Ok(())
}

/// Main shell UI loop. Collects input from the user, conditionally consults LLMs depending on the user prompt, executes
/// subsequent commands and updates shell state.
pub fn
shell_loop (context: &mut Context, model: Box<dyn Model>) -> Result<(), Box<dyn std::error::Error>>
{
  loop {
    // Define the prompt prefix string, something like
    // [nl-sh] /Users/mike $
    let terminal_prompt = std::format!("[nl-sh] {} $", context.pwd.as_str());

    // Collect the user input from the prompt; update the prompt history from the context
    let input = Text::new(&terminal_prompt)
      .with_history(SimpleHistory::new(context.get_command_history()))
      .prompt();

    match input {
      Ok(input) => {
        if input.is_empty() {
          continue;
        }

        // If the input is a likely and unambiguous system command, we'll take the text as-is and exec it through the shell.
        // Otherwise, we'll pass the input to the model and let the LLM sort it out. If it is, in fact, a valid
        // command and argument, the model should return the input string.
        let cmd = if likely_system_command(context, &input) {
          input.clone() 
        } else {
          // Fetch input rationalization from the model
          model.ask_model(context, &input)?
        };

        let confirm = if input.eq(&cmd) {
          // If the input from the user is identical to the command to execute, just execute it without 
          // asking for confirmation from the shell user.
          Ok(true) 
        } else if cmd.trim().is_empty() {
          // If the command string is empty, this means the model didn't consider the input to be a sensible
          // shell command.
          println!("could not interpret request");
          Ok(true)
        } else {
          // Confirm with the user that they would like to execute the command
          Confirm::new(&cmd)
            .with_default(true)
            .with_help_message("execute this command?")
            .prompt()
        };

        match confirm {
          Ok(true) => {
            // Execute the command, by passing it to `$SHELL -c <command string>`
            let output = Command::new(context.shell.clone()) // run in env shell
              .arg("-c")
              .arg(cmd.clone())
              .output()
              .expect("failed to execute command");

            if output.status.success() {
              println!("{}", std::str::from_utf8(&output.stdout).expect("failed to convert stdout to String"));
            } else {
              println!("Executed [{}] and got error: {}", 
                cmd, std::str::from_utf8(&output.stderr).expect("failed to convert stdout to String"));
            }
          },
          Ok(false) => {
            println!("Aborting command")
          },
          Err(e) if matches!(e, InquireError::OperationCanceled) || matches!(e, InquireError::OperationInterrupted) => {
            println!("exiting");
            break Ok(());
          },
          Err(e) => {
            println!("error: {}", e);
            break Ok(());
          }
        }

        maybe_update_context(&cmd, context)?;
      }, // Ok(input)
    Err(e) if matches!(e, InquireError::OperationCanceled) || matches!(e, InquireError::OperationInterrupted) => {
      // This was a ^C or esc
      println!("exiting");
      break Ok(());
    },
    Err(e) => {
      println!("exiting: {}", e);
      break Ok(());
    }
   }
  }
}