use clap::{Arg, ArgAction, Command as CommandArg};

use nl_sh::*;
pub use crate::anthropic::*;
pub use crate::args::*;
pub use crate::context::*;
pub use crate::local::*;
pub use crate::model::*;
pub use crate::shell::*;

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
      .help("Use the Anthropic Claude API as a backend (default: Claude 3 Sonnet), reading from the CLAUDE_API_KEY environment variable"))
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
    ModelType::Claude => Box::new(Claude { version: claude_3_sonnet(), client: anthropic_client() }),
    ModelType::Local(ref path) => Box::new(LocalLLM { local: local_llm(path) }),
  };

  let executor = CommandExecutor {};
  let mut context = Context::init(&args, &executor, model.as_ref())?;

  shell_loop(&mut context, model, &executor)?;
  Ok(())
}