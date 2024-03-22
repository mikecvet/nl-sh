use openai_api_rs::v1::api::Client as OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::{GPT3_5_TURBO, GPT4};
use std::env;

pub fn
gpt4_version () -> String
{
  GPT4.to_string()
}

pub fn
gpt35_version () -> String
{
  GPT3_5_TURBO.to_string()
}

pub fn
open_ai_api_client () -> OpenAIClient
{
  match env::var("OPENAI_API_KEY") {
    Ok(key) => OpenAIClient::new(key.to_string()),
    Err(e) => panic!("OPENAI_API_KEY must be set as an environment variable in order to issue requests to OpenAI APIs: {e}")
  }
}

pub fn
issue_open_ai_request (client: &OpenAIClient, model: String, prompt: &str) -> Result<String, Box<dyn std::error::Error>>
{
  let req = ChatCompletionRequest::new(
    model,
    vec![chat_completion::ChatCompletionMessage {
        role: chat_completion::MessageRole::user,
        content: chat_completion::Content::Text(String::from(prompt)),
        name: None,
    }],
  );

  let response = client.chat_completion(req)?;
  match response.choices[0].message.content.clone() {
    Some(message) => Ok(message.trim_matches('"').to_string()),
    _ => Ok("".to_string())
  }
}