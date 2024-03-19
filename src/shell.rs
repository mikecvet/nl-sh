use inquire::{Confirm, Text};
use inquire::error::InquireError;
use inquire::history::SimpleHistory;

pub use crate::command::*;
pub use crate::context::*;
pub use crate::model::*;

#[cfg(test)]
use mockall::predicate::*;

/// Some POSIX commands have verb-y characteristics, and for these, we'll let the LLM determine
/// whether the user intent is to run a specfic command, or whether the request is something
/// akin to "diff these two files" or "sort and print the text in a, b, and c"
static COMMAND_EXCEPTIONS: &[&str] = &["alias", "cat", "diff", "expand", "find", "kill", "link", 
  "list", "log", "print", "read", "sort", "split", "strip", "touch", "type", "what", "which", "who"];

/// Determine whether the input from the prompt is a likely system command. This is
/// a best-effort attempt to short-circut requests to an LLM, to improve performance
/// for the most obvious native command interpreted by the prompt.
/// 
/// In the case where the user request is ambiguous, as defined by the set 
/// of COMMAND_EXCEPTIONS strings, fall through and indicate to pass the user input
/// to the LLM to rationalize.
fn 
likely_system_command (context: &Context, command: &String, executor: &dyn CommandExecutorInterface) -> bool 
{
  let parts: Vec<&str> = command.split_whitespace().collect();
  let cmd = parts.get(0).unwrap_or(&"").to_lowercase(); // Extract the just command without arguments

  if COMMAND_EXCEPTIONS.contains(&cmd.as_str()) {
    // The user string contains a verb-y command, let the LLM sort it out
    return false
  } else {
    // Determine whether the first word in the user input is valid command on this system, 
    // via $SHELL command -v <cmd>
    executor.exists(&context.shell, command)
  }
}

/// Main shell UI loop. Collects input from the user, conditionally consults LLMs depending on the user prompt, executes
/// subsequent commands and updates shell state.
pub fn
shell_loop (context: &mut Context, model: Box<dyn Model>) -> Result<(), Box<dyn std::error::Error>>
{
  let executor = CommandExecutor {};

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
        let system_command = likely_system_command(context, &input, &executor);
        let mut cmd = if system_command {
          input.clone() 
        } else {
          // Fetch input rationalization from the model
          model.ask_model(context, &input)?
        };

        // The following runs in a simple loop, allowing for a single retry of a failed system command, by requesting a
        // command correction from the model given context about the command objective and failure output.
        for i in 0..2 {
          let confirm = if input.eq(&cmd) {
            // If the input from the user is identical to the command to execute, just execute it without 
            // asking for confirmation from the shell user.
            Ok(true) 
          } else if cmd.trim().is_empty() {
            // If the command string is empty, this means the model didn't consider the input to be a sensible
            // shell command.
            println!("\ncould not interpret request");
            continue;
          } else {
            if i == 0 {
              print!("\n");
            }

            // Confirm with the user that they would like to execute the command
            Confirm::new(&cmd)
              .with_default(true)
              .with_help_message("execute this command?")
              .prompt()
          };

          match confirm {
            Ok(true) => {
                // Execute the confirmed command string on the system
                let output = executor.execute(&context.shell, &cmd)?;

                if output.success {
                  // If successful, emit the stdout captured by the command
                  print!("\n{}", output.stdout);

                  // Update the context state based on the issued command
                  context.update(&cmd)?;
                  break;
                } else {
                  println!("Executed [{}] and got error: {}", cmd, output.stderr);

                  // If this wasn't a system command, then see if we can collect a correction from the model. If it /was/ a system command,
                  // assume that the operator is trying to enter some complex commands themselves and don't bother trying to fetch corrections.
                  if !system_command {
                    if i == 0 {
                      println!("Retrying command formulation...");
                    }

                    cmd = model.attempt_correction(context, &input.as_str(), &cmd, &output)?;
                  } else {
                    break;
                  }
                }
            },
            Ok(false) => {
              println!("Aborting command")
            },
            Err(e) if matches!(e, InquireError::OperationCanceled) || matches!(e, InquireError::OperationInterrupted) => {
              println!("exiting");
              return Ok(());
            },
            Err(e) => {
              println!("error: {}", e);
              return Ok(());
            }
          }
      }
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

#[cfg(test)]
mod tests 
{
  use super::*;

  fn
  get_test_context() -> Context 
  {
    Context {
      uname: "Darwin".to_string(),
      shell: "/bin/zsh".to_string(),
      os: "Darwin 23.3.0 arm64".to_string(),
      pwd: "/home".to_string(),
      history: CommandHistory::init("/bin/zsh", false).unwrap(),
    }
  }

  #[test]
  fn test_likely_system_command() 
  {
    let mut mock_executor = MockCommandExecutorInterface::new();
    mock_executor.expect_exists()
      .with(eq("/bin/zsh"), eq("ls"))
      .returning(|_, _| true);

    let context = get_test_context();

    assert!(likely_system_command(&context, &"ls".to_string(), &mock_executor));
  }

  #[test]
  fn test_not_likely_system_command() 
  {
    let mut mock_executor = MockCommandExecutorInterface::new();
    mock_executor.expect_exists()
      .with(eq("/bin/zsh"), eq("ls"))
      .returning(|_, _| false);

    let context = get_test_context();

    assert!(!likely_system_command(&context, &"alias".to_string(), &mock_executor));
  }
}
