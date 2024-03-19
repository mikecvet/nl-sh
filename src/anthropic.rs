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
  match env::var("ANTHROPIC_API_KEY") {
    Ok(key) => ClientBuilder::default().api_key(key.to_string()).build().unwrap(),
    Err(e) => panic!("ANTHROPIC_API_KEY must be set as an environment variable in order to issue requests to Anthropic APIs: {e}")
  }
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
  
  match client.complete(req).await {
    Ok(response) => Ok(response),
    Err(e) => {
      println!("Anthropic API error: {e}");
      Err(e)
    }
  }
}