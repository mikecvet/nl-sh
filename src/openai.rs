use openai_api_rs::v1::api::Client as OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, ChatCompletionResponse};
use openai_api_rs::v1::common::GPT4;
use openai_api_rs::v1::error::APIError;
use std::env;

pub fn 
gpt4_client () -> OpenAIClient 
{
  OpenAIClient::new(env::var("OPENAI_API_KEY").unwrap().to_string())
}

pub fn
issue_gpt4_request (client: &OpenAIClient, prompt: &str) -> Result<ChatCompletionResponse, APIError>
{
  let req = ChatCompletionRequest::new(
    GPT4.to_string(),
    vec![chat_completion::ChatCompletionMessage {
        role: chat_completion::MessageRole::user,
        content: chat_completion::Content::Text(String::from(prompt)),
        name: None,
    }],
  );

  client.chat_completion(req)
}