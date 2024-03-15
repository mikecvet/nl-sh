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
    let s = issue_local_llm_request(&self.local, &build_init_prompt(input))?;
    Ok(s)
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
build_command_prompt (context: &Context, arg: &str) -> String 
{
  format!(
    "Output the command-line arguments to satisfy the following prompt. 
    The underlying kernel details according to \"uname -smr\" is [{}] and operating system details include 
      [{}].
    The underlying shell is [{}]. The current working directory is [{}]. 
    Respond only with the proper command-line details to satisfy the request, without any additional context or explanation. 
    Be terse. This command will be directly executed on this system.
    The returned command and arguments must be a valid command on this operating system, 
    since not all POSIX platforms have the same command or argument sets and syntax.
    For example, on GNU/Linux `ps -aux` is valid, however on Darwin OS the equivalent command is `ps aux`.
    If the prompt is already a valid POSIX command, then just return the original input.
    If the prompt is an incoherent request for a POSIX-style command, return an empty string.
    Here is the prompt: \"{}\"", context.uname, context.os, context.shell, context.pwd, arg
  )
}

fn 
build_init_prompt (arg: &str) -> String 
{
  format!(
    "Given this output from the POSIX command `uname - smr`, provide the best next command to run within a shell to 
    get specific details of the underlying operating system variant and version. Return only the command with no additional explanation or context. Be terse.
    This should not be a script, but a simple shell command which is directly executable. For example, on Darwin OS, an appropriate command might be simply `sw_vers`.
    Here is the uname output: {}", arg
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
    Given that, suggest an updated command given the constraints of this system's environment. Follow all earlier instructions; specifically, emit only the command with no additional context or explanation", 
      build_command_prompt(context, arg),
      command,
      output.status_code, output.stderr
  )
}
