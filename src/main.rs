use clap::{arg, Command as CommandArg};
use std::env;
use std::process::Command;

use nl_sh::*;
pub use crate::context::*;
pub use crate::llama::*;
pub use crate::model::*;
pub use crate::shell::*;

/// Initializes this instance's `Context`, by issuing a preliminary request to the `Model` 
/// asking for the best next command to gather local OS and environment information, given
/// the content of a call to `uname`
fn 
initialize_env_context (model: &Box<dyn Model>) -> Result<Context, Box<dyn std::error::Error>>
{
  let uname_output = Command::new("uname")
    .args(&["-smr"])
    .output()
    .expect("failed to execute command");

  let os = std::str::from_utf8(&uname_output.stdout).expect("failed to convert uname stdout to String");

  let os_response = model.init_prompt(os)?;

  let mut args: Vec<String> = os_response.split_whitespace().map(String::from).collect();
  let os_command = args.remove(0);

  let os_output = Command::new(os_command.clone())
    .args(&args)
    .output()
    .expect("failed to execute command");

  let shell_output = env::var("SHELL");

  match (os_output.status.success(), shell_output) {
    (true, Ok(shell)) => {
      Ok(Context::init(uname_output.stdout, &shell, os_output.stdout)?)
    },
    (os_success, _) => {
      panic!("failed to collect outputs {}", os_success);
    }
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> 
{
  let matches = CommandArg::new("nl-sh")
  .version("0.1")
  .about("A natural language shell for *NIX systems")
  .arg(arg!(--gpt4)
    .required(false)
    .value_name("BOOL")
    .help("Use the GPT4 API as a backend, reading from the OPENAI_API_KEY environment variable"))
  .arg(arg!(--llama <VALUE>)
    .required(false)
    .value_name("path")
    .help("Use a local Llama model as a backend, located at the provided path"))
  .get_matches();

  let gpt4_opt = matches.get_one::<bool>("gp4").cloned();
  let llama_opt = matches.get_one::<String>("llama").cloned();

  let model: Box<dyn Model> = match (gpt4_opt, llama_opt) {
    (Some(true), _) => Box::new(GPT4 { client: gpt4_client() }),
    (_, None) => Box::new(GPT4 { client: gpt4_client() }),
    (_, Some(path)) => Box::new(LLama2 { llama: llama(&path) })
  };

  let mut context = initialize_env_context(&model)?;

  shell_loop(&mut context, model)?;
  Ok(())
}