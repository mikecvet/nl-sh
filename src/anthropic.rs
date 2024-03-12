use anthropic::client::{Client as AnthropicClient, ClientBuilder};
use anthropic::error::AnthropicError;
use anthropic::types::{CompleteRequestBuilder, CompleteResponse};
use anthropic::{AI_PROMPT, HUMAN_PROMPT};
use std::env;

pub fn 
claude_version () -> String
{
  "claude-2.1".to_string()
}

pub fn 
anthropic_client () -> AnthropicClient
{
  ClientBuilder::default().api_key(env::var("ANTHROPIC_API_KEY").unwrap().to_string()).build().unwrap()
}

pub async fn
issue_anthropic_request (client: &AnthropicClient, model: String, prompt: &str) -> Result<CompleteResponse, AnthropicError>
{
  let req = CompleteRequestBuilder::default()
    .prompt(format!("{HUMAN_PROMPT}{prompt}{AI_PROMPT}"))
    .model(model)
    .max_tokens_to_sample(256usize)
    .stream(false)
    .stop_sequences(vec![HUMAN_PROMPT.to_string()])
    .build()?;

  client.complete(req).await
}