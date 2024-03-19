use clap::{Arg, ArgAction, Command as CommandArg};
use std::env;
use std::process::Command;

use nl_sh::*;
pub use crate::anthropic::*;
pub use crate::args::*;
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
  let os_response = match model.init_prompt(os) {
    Ok(response) => response,
    Err(e) => panic!("Failed to initialize environment context due to model error: {e}")
  };

  let mut args: Vec<String> = os_response.split_whitespace().map(String::from).collect();
  let os_command = args.remove(0);

  let os_output = Command::new(os_command.clone())
    .args(&args)
    .output()
    .expect(format!("failed to execute command: {os_command} {:?}", args).as_str());

  let shell_output = env::var("SHELL");

  match (os_output.status.success(), shell_output) {
    (true, Ok(shell)) => {
      Ok(Context::init(uname_output.stdout, &shell, os_output.stdout, stateless)?)
    },
    (_, Err(e)) => {
      panic!("failed to determine shell: {e}");
    },
    (os_success, _) => {
      panic!("failed to collect outputs {os_success}");
    }
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> 
{
  let matches = CommandArg::new("nl-sh")
    .version("0.21")
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

  let args = Args::new(&matches);
  let model: Box<dyn Model> = match args.model_type {
    ModelType::GPT4 => Box::new(GPT { version: gpt4_version(), client: open_ai_api_client() }),
    ModelType::GPT35 => Box::new(GPT { version: gpt35_version(), client: open_ai_api_client() }),
    ModelType::Claude => Box::new(Claude { version: claude_version(), client: anthropic_client() }),
    ModelType::Local(ref path) => Box::new(LocalLLM { local: local_llm(path) }),
  };

  let mut context = initialize_env_context(&model, args.stateless)?;

  shell_loop(&mut context, model)?;
  Ok(())
}