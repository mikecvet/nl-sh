use clap::{Arg, ArgAction, Command as CommandArg, Parser};
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
    .expect("failed to execute command");

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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  #[clap(long, action, action=ArgAction::SetTrue)]
  gpt4: bool,

  #[clap(long)]
  llama: String,

  #[clap(long, action, action=ArgAction::SetFalse)]
  stateless: bool
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
      .help("Use the GPT4 API as a backend, reading from the OPENAI_API_KEY environment variable"))
    .arg(Arg::new("llama")
      .long("llama")
      .value_name("path")
      .help("Use a local Llama model as a backend, located at the provided path"))
    .arg(Arg::new("stateless")
      .long("stateless")
      .action(ArgAction::SetTrue)
      .default_value("false")
      .help("Disable update of external shell history (default: false)"))
    .get_matches();
  
  let mut gpt4 = matches.get_one::<bool>("gpt4").map(|&b| b).unwrap_or(true);
  let llama_opt = matches.get_one::<String>("llama").cloned();
  let stateless = matches.get_one::<bool>("stateless").map(|&b| b).unwrap_or(false);

  if llama_opt.is_some() {
    gpt4 = false;
  }

  let model: Box<dyn Model> = match (gpt4, llama_opt) {
    (true, _) => Box::new(GPT4 { client: gpt4_client() }),
    (_, None) => Box::new(GPT4 { client: gpt4_client() }),
    (_, Some(path)) => Box::new(LLama2 { llama: llama(&path) })
  };

  let mut context = initialize_env_context(&model, stateless)?;

  shell_loop(&mut context, model)?;
  Ok(())
}