use openai_api_rs::v1::api::Client as OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, ChatCompletionResponse};
use openai_api_rs::v1::common::GPT4;
use openai_api_rs::v1::error::APIError;

pub fn
issue_gpt4_request (client: &OpenAIClient, prompt: &str) -> Result<ChatCompletionResponse, APIError>
{
  //println!("issue request with prompt: [{}]", prompt);
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