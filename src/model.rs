use llama_cpp_rs::LLama;
use openai_api_rs::v1::api::Client as OpenAIClient;

pub use crate::context::*;
pub use crate::llama::*;
pub use crate::openai::*;

pub trait Model {
  /// Main query interface; uses the command prompt to collect NIX commands given the user string
  fn ask_model (&self, context: &Context, input: &str) -> Result<String, Box<dyn std::error::Error>>;

  /// Used strictly for initialization of local context with information used to construct a better command query
  fn init_prompt (&self, input: &str) -> Result<String, Box<dyn std::error::Error>>;
}

pub struct GPT {
  pub version: String,
  pub client: OpenAIClient
}

pub struct LocalLLM {
  pub local: LLama
}

/// Constructs a prompt given current environment context, and issues a requet to OpenAI's GPT4 via their API client.
impl Model for GPT {
  fn
  ask_model (&self, context: &Context, input: &str) -> Result<String, Box<dyn std::error::Error>>
  {
    self.request(&build_command_prompt(context, input))
  }

  fn
  init_prompt (&self, input: &str) -> Result<String, Box<dyn std::error::Error>>
  {
    self.request(&build_init_prompt(input))
  }
}

impl GPT {
  fn 
  request (&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> 
  {
    let result = issue_open_ai_request(&self.client, self.version.clone(), prompt)?;

    match result.choices[0].message.content.clone() {
      Some(message) => Ok(message),
      _ => Ok("".to_string())
    }
  }
}

/// Constructs a prompt given current environment context, and issues a requet to a local Llama model
impl Model for LocalLLM {
  fn 
  ask_model (&self, context: &Context, input: &str) -> Result<String, Box<dyn std::error::Error>> {
    issue_local_llm_request(&self.local, &build_command_prompt(context, input))
  }

  fn 
  init_prompt (&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
    issue_local_llm_request(&self.local, &build_init_prompt(input))
  }
}

fn 
build_command_prompt (context: &Context, arg: &str) -> String 
{
  format!(
    "Output the command-line arguments to satisfy the following prompt. 
    The underlying kernel details according to \"uname -smr\" is [{}] and operating system details include 
      [{}].
    The underlying shell is [{}]. The current working directory is [{}]. 
    Respond only with the proper command-line details to satisfy the request, without any additional context or explanation.
    The returned command and arguments must be a valid command on this operating system, since not all POSIX platforms have the same command or argument sets and syntax.
    For example, on GNU/Linux `ps -aux` is valid, however on Mac OS Darwin the equivalent command is `ps aux`.
    If the prompt is already a valid POSIX command, then just return the original input.
    If the prompt is an incoherent request for a POSIX-style command, return an empty string.
    Here is the prompt: {}", context.uname, context.os, context.shell, context.pwd, arg
  )
}

fn 
build_init_prompt (arg: &str) -> String 
{
  format!(
    "Given this output from the POSIX command `uname - smr`, provide the best next command to run within a shell to 
    get specific details of the underlying operating system variant and version. Return only the command with no additional explanation or context. 
    This should not be a script, but a simple command-line command which is directly executable. For example, on Mac OS, an appropriate command might be simply `sw_vers`.
    Here is the uname output: {}", arg
  )
}
