use clap::{Arg, ArgAction, Command as CommandArg};
use std::env;
use std::process::Command;

use nl_sh::*;
pub use crate::anthropic::*;
pub use crate::context::*;
pub use crate::local::*;
pub use crate::model::*;
pub use crate::shell::*;

/// Initializes this instance's `Context`, by issuing a preliminary request to the `Model` 
/// asking for the best next command to gather local OS and environment information, given
/// the content of a call to `uname`
fn 
initialize_env_context (model: &Box<dyn Model>, stateless: bool) -> Result<Context, Box<dyn std::error::Error>>
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
    .expect(format!("failed to execute command: {} {:?}", os_command, args).as_str());

  let shell_output = env::var("SHELL");

  match (os_output.status.success(), shell_output) {
    (true, Ok(shell)) => {
      Ok(Context::init(uname_output.stdout, &shell, os_output.stdout, stateless)?)
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
    .arg(Arg::new("gpt4")
      .long("gpt4")
      .action(ArgAction::SetTrue)
      .default_value("true")
      .help("Use the GPT4 API as a backend, reading from the OPENAI_API_KEY environment variable. Default behavior."))
    .arg(Arg::new("gpt35")
      .long("gpt35")
      .action(ArgAction::SetTrue)
      .default_value("false")
      .help("Use the GPT3.5 Turbo API as a backend, reading from the OPENAI_API_KEY environment variable"))
    .arg(Arg::new("claude")
      .long("claude")
      .action(ArgAction::SetTrue)
      .default_value("false")
      .help("Use the Anthropic Claude API as a backend, reading from the CLAUDE_API_KEY environment variable"))  
    .arg(Arg::new("local")
      .long("local")
      .value_name("path")
      .help("Use a local GGUF-based model as a backend, located at the provided path"))
    .arg(Arg::new("stateless")
      .long("stateless")
      .action(ArgAction::SetTrue)
      .default_value("false")
      .help("Disable update of external shell history (default: false)"))
    .get_matches();
  
  let mut gpt4 = matches.get_one::<bool>("gpt4").map(|&b| b).unwrap_or(true);
  let gpt35 = matches.get_one::<bool>("gpt35").map(|&b| b).unwrap_or(false);
  let claude = matches.get_one::<bool>("claude").map(|&b| b).unwrap_or(false);
  let local_opt = matches.get_one::<String>("local").cloned();
  let stateless = matches.get_one::<bool>("stateless").map(|&b| b).unwrap_or(false);

  if local_opt.is_some() || gpt35 || claude {
    gpt4 = false;
  }

  let model: Box<dyn Model> = match (gpt4, gpt35, claude, local_opt) {
    (true, _, _, _) => Box::new(GPT { version: gpt4_version(), client: open_ai_api_client() }),
    (false, true, _, _) => Box::new(GPT { version: gpt35_version(), client: open_ai_api_client() }),
    (false, false, true, _) => Box::new(Claude { version: claude_version(), client: anthropic_client() }),
    (false, false, false, Some(path)) => Box::new(LocalLLM { local: local_llm(&path) }),
    (false, false, false, None) => panic!("no model specified")
  };

  let mut context = initialize_env_context(&model, stateless)?;

  shell_loop(&mut context, model)?;
  Ok(())
}