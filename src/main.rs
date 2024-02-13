use openai_api_rs::v1::api::Client as OpenAIClient;
use openai_api_rs::v1::error::APIError;
use std::env;
use std::process::Command;

use nl_sh::*;
pub use crate::context::*;
pub use crate::model::*;
pub use crate::openai::*;
pub use crate::shell::*;

const INIT_PROMPT_TEMPLATE: &'static str = "Given this output from the POSIX command `uname - smr`, provide the best next command to run to 
  get specific details of the underlying operating system variant and version. Return only the command with no additional explanation or context. 
  Here is the uname output: ";

fn 
initialize_env_context (client: &OpenAIClient) -> Result<Context, APIError>
{
  let uname_output = Command::new("uname")
    .args(&["-smr"])
    .output()
    .expect("failed to execute command");

  let os = std::str::from_utf8(&uname_output.stdout).expect("failed to convert uname stdout to String");
  let os_prompt = format!("{} {}", INIT_PROMPT_TEMPLATE, os);

  let result = issue_gpt4_request(&client, &os_prompt)?;
  let os_response = result.choices[0].message.content.clone().unwrap();

  let mut args: Vec<String> = os_response.split_whitespace().map(String::from).collect();
  let os_command = args.remove(0);

  let os_output = Command::new(os_command.clone())
    .args(&args)
    .output()
    .expect("failed to execute command");

  let shell_output = env::var("SHELL");

  match (os_output.status.success(), shell_output) {
    (true, Ok(shell)) => {
      Ok(Context::init(uname_output.stdout, &shell, os_output.stdout))
    },
    (os_success, _) => {
      panic!("failed to collect outputs {}", os_success);
    }
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> 
{
  let client = OpenAIClient::new(env::var("OPENAI_API_KEY").unwrap().to_string());
  let mut context = initialize_env_context(&client)?;
  let model = GPT4 { client: client };

  shell_loop(&mut context, &model)?;
  Ok(())
}