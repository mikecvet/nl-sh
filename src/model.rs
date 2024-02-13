use openai_api_rs::v1::api::Client as OpenAIClient;

pub use crate::context::*;
pub use crate::openai::*;

pub trait Model {
  fn ask_model (&self, context: &Context, input: &str) -> Result<String, Box<dyn std::error::Error>>;
}

pub struct GPT4 {
  pub client: OpenAIClient
}

/// Constructs a prompt given current environment context, and issues a requet to OpenAI's GPT4 via their API client.
impl Model for GPT4 {
  fn
  ask_model (&self, context: &Context, input: &str) -> Result<String, Box<dyn std::error::Error>>
  {
    let prompt = &construct_prompt(context, input);
    let result = issue_gpt4_request(&self.client, prompt)?;

    match result.choices[0].message.content.clone() {
      Some(message) => Ok(message),
      _ => Ok("".to_string())
    }
  }
}

fn 
construct_prompt (context: &Context, arg: &str) -> String 
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
