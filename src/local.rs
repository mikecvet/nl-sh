use llama_cpp_rs::LLama;
use llama_cpp_rs::options::{ModelOptions, PredictOptions};
use regex::Regex;

pub fn 
local_llm (model_path: &str) -> LLama
{
  let mut model_options = ModelOptions::default();
  model_options.set_gpu_layers(1);

  LLama::new(
    model_path.into(),
    &model_options,
  ).unwrap()
}

pub fn 
issue_local_llm_request (local_llm: &LLama, prompt: &str) -> Result<String, Box<dyn std::error::Error>>
{
  let options = PredictOptions::default();

  match local_llm.predict(prompt.into(), options) {
    Ok(text) => {
      Ok(extract_command(&text).unwrap_or("".to_string()))
    },
    e => e
  }
}

/// Often, local models are "chatty" and return a lot of extra context, even when instructed not to. As an
/// example, here's this shell's init prompt response from a local Mistral model:
/// 
/// """
/// >>> Given this output from the POSIX command `uname - smr`, provide the best next command to run within a shell to
///     get specific details of the underlying operating system variant and version. Return only the command with no additional explanation or context.
///     This should not be a script, but a simple command-line command which is directly executable. 
///     For example, on Mac OS, an appropriate command might be simply `sw_vers`.
///     Here is the uname output: Darwin 23.3.0 arm64
/// 
/// Based on the given `uname` output being "Darwin 23.3.0 arm64", a suitable command to obtain more details of the MacOS variant and version would be:
///
/// ```bash
/// sw_vers -productName -version
/// ```
///
/// This command displays the name and version number of the installed operating system on MacOS.
/// """
/// 
/// It turns out that several open-source models return the command we want, captured within a triple-backticks block
/// such as above. This function attempts to detect and extract this pattern for use by the shell.
fn
extract_command (text: &str) -> Option<String> 
{
  let re = Regex::new(r"```[a-zA-Z]*\n([\s\S]*?)```").unwrap();

  if let Some(caps) = re.captures(text) {
    caps.get(1).map(|matched| matched.as_str().trim().to_string())
  } else {
    Some(text.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn basic_command_extraction() {
    let text = "```\nls -la\n```";
    assert_eq!(extract_command(text), Some("ls -la".to_string()));
  }

  #[test]
  fn command_extraction_with_language_specifier() {
    let text = "```bash\nls -la\n```";
    assert_eq!(extract_command(text), Some("ls -la".to_string()));
  }

  #[test]
  fn text_without_backticks() {
    let text = "This is a regular text.";
    assert_eq!(extract_command(text), Some(text.to_string()));
  }

  #[test]
  fn multiple_commands() {
    let text = "```bash\necho 'Hello'\n```\n```zsh\nls\n```";
    assert_eq!(extract_command(text), Some("echo 'Hello'".to_string()));
  }

  #[test]
  fn commands_with_additional_newlines() {
    let text = "```bash\n\nls -la\n\n```";
    assert_eq!(extract_command(text), Some("ls -la".to_string()));
  }

  #[test]
  fn empty_command() {
    let text = "```\n```";
    assert_eq!(extract_command(text), Some("".to_string()));
  }

  #[test]
  fn invalid_cases() {
    // Expect the original text because it doesn't match the pattern
    let text = "```bashls -la";
    assert_eq!(extract_command(text), Some(text.to_string()));
  }
}
