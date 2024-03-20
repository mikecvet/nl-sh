use anthropic::client::Client as AnthropicClient;
use llama_cpp_rs::LLama;
use openai_api_rs::v1::api::Client as OpenAIClient;
use tokio::runtime::Runtime;

pub use crate::anthropic::*;
pub use crate::command::*;
pub use crate::context::*;
pub use crate::local::*;
pub use crate::openai::*;

pub trait Model {
  /// Used strictly for initialization of local context with information used to construct a better command query
  fn init_prompt (&self, input: &str) -> Result<String, Box<dyn std::error::Error>>;

  /// Main query interface; uses the command prompt to collect NIX commands given the user string
  fn ask_model (&self, context: &Context, input: &str) -> Result<String, Box<dyn std::error::Error>>;

  /// A previously-suggested command failed. Provide the failure context back to the model and attempt a correction to the command
  fn attempt_correction (&self, context: &Context, input: &str, command: &str, output: &CommandOutput) -> Result<String, Box<dyn std::error::Error>>;
}

pub struct GPT {
  pub version: String,
  pub client: OpenAIClient
}

pub struct LocalLLM {
  pub local: LLama
}

pub struct Claude {
  pub version: String,
  pub client: AnthropicClient
}

/// Constructs a prompt given current environment context, and issues a requet to OpenAI's GPT4 via their API client.
impl Model for GPT {
  fn
  init_prompt (&self, input: &str) -> Result<String, Box<dyn std::error::Error>>
  {
    self.request(&build_init_prompt(input))
  }

  fn
  ask_model (&self, context: &Context, input: &str) -> Result<String, Box<dyn std::error::Error>>
  {
    self.request(&build_command_prompt(context, input))
  }

  fn 
  attempt_correction (&self, context: &Context, input: &str, command: &str, output: &CommandOutput) -> Result<String, Box<dyn std::error::Error>>
  {
    self.request(&build_correction_prompt(context, input, command, output))
  }
}

impl GPT {
  fn 
  request (&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> 
  {
    let result = issue_open_ai_request(&self.client, self.version.clone(), prompt)?;

    match result.choices[0].message.content.clone() {
      Some(message) => Ok(message.trim_matches('"').to_string()),
      _ => Ok("".to_string())
    }
  }
}

/// Constructs a prompt given current environment context, and issues a requet to a local Llama model
impl Model for LocalLLM {
  fn 
  init_prompt (&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
    issue_local_llm_request(&self.local, &build_init_prompt(input))
  }

  fn 
  ask_model (&self, context: &Context, input: &str) -> Result<String, Box<dyn std::error::Error>> {
    issue_local_llm_request(&self.local, &build_command_prompt(context, input))
  }

  fn 
  attempt_correction (&self, context: &Context, input: &str, command: &str, output: &CommandOutput) -> Result<String, Box<dyn std::error::Error>>
  {
    issue_local_llm_request(&self.local, &build_correction_prompt(context, input, command, output))
  }
}

impl Model for Claude {
  fn
  init_prompt (&self, input: &str) -> Result<String, Box<dyn std::error::Error>>
  {
    self.request(&build_init_prompt(input))
  }

  fn
  ask_model (&self, context: &Context, input: &str) -> Result<String, Box<dyn std::error::Error>>
  {
    self.request(&build_command_prompt(context, input))
  }

  fn 
  attempt_correction (&self, context: &Context, input: &str, command: &str, output: &CommandOutput) -> Result<String, Box<dyn std::error::Error>>
  {
    self.request(&build_correction_prompt(context, input, command, output))
  }
}

impl Claude {
  fn 
  request (&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> 
  {
    match Runtime::new()?.block_on(issue_anthropic_request(&self.client, self.version.clone(), prompt)) {
      Ok(response) => Ok(response.completion.trim_matches('"').to_string()),
      Err(e) => Err(Box::new(e))
    }
  }
}

fn 
build_init_prompt (arg: &str) -> String 
{
  format!(
    "You are being asked to provide a POSIX-compatible command line sequence to gather information about the underlying operating system variant and version.
    The underlying details according to the command \"uname -smr\" include:
      \"{arg}\"
    Respond only the specific command-line details to satisfy the request, with no additional context or explanation. Be terse and exact.
    Since commands differ on various *NIX systems, ensure the command is valid in the environment detailed above.
    For example: 
      On a Darwin UNIX system, an appropriate command might be \"sw_vers\". 
      On a GNU/Linux system, the right command might be \"hostnamectl\" or \"cat /etc/os-release\".
    Provide the command."
  )
}

fn 
build_command_prompt (context: &Context, arg: &str) -> String 
{
  format!(
    "You are being asked to provide a POSIX-compatible command line sequence to satisfy a user's prompt.
    The command line arguments should be compatible with the user's operating system.
    The underlying kernel and system details according to \"uname -smr\" includes \"{}\"
    Further operating systems details include \"{}\"
    The user's underlying shell is \"{}\"
    The user's current working directory according to \"pwd\" is \"{}\"
    Respond only with the specific command-line details to satisfy the request, with no additional context or explanation. Be terse and exact.
    Since commands differ on various *NIX systems, ensure the command is valid in the environment detailed above.
    Here are a few examples, on a Darwin-based UNIX system:
      User: \"Show me deteails about all running processes on this system\"
        Your response: \"ps aux\"
      User: \"Show me all files in the current directory with human-readable file sizes and permission details\"
        Your response: \"ls -lha\"
      User: \"Show me a summary of Mike's commits in this git repository; show line additions and subtractions to each file in each commit\"
        Your response: \"git log --stat --summary --author='Mike'\"
    If the prompt is already a valid *NIX command for the user's system, then just return the original input.
    If the prompt is an incoherent request for a POSIX-style command, return an empty string.
    If the prompt is a command sequence for a different *NIX system, return the right combination of commands and flags to satisfy the request on the current system.
    If the user's intention requires superuser priviledges, ensure to prefix the command with 'sudo' or an appropriate equivalent given the operating system.
    Here is the user's prompt: 
      \"{arg}\"", context.uname, context.os, context.shell, context.pwd
  )
}

fn 
build_correction_prompt (context: &Context, arg: &str, command: &str, output: &CommandOutput) -> String
{
  format!(
    "In an earlier conversation, the following prompt was given: 
    {}
    \nThis resulted in the following proposed command: {}
    Executing that proposal on this system resulted in failure, with this status code: \"{}\" and this stderr output: \"{}\"
    Given that, suggest an updated command given the constraints of this system's stated environment and the intent of the user. 
    Follow all earlier instructions; specifically, emit only the command with no additional context or explanation", 
      build_command_prompt(context, arg),
      command,
      output.status_code, output.stderr
  )
}
