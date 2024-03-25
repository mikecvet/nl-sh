use clust::Client as AnthropicClient;
use clust::messages::*;

pub fn 
claude_3_haiku () -> ClaudeModel
{
  ClaudeModel::Claude3Haiku20240307
}

pub fn 
claude_3_sonnet () -> ClaudeModel
{
  ClaudeModel::Claude3Sonnet20240229
}

pub fn 
claude_3_opus () -> ClaudeModel
{
  ClaudeModel::Claude3Opus20240229
}

pub fn 
anthropic_client () -> AnthropicClient
{
  // Expects ANTHROPIC_API_KEY to be set in environment
  AnthropicClient::from_env().unwrap()
}

pub async fn
issue_anthropic_request (client: &AnthropicClient, model: ClaudeModel, prompt: &str) -> Result<String, Box<dyn std::error::Error>>
{
  let messages = vec![Message::user(prompt)];
  let max_tokens = MaxTokens::default();
  let request_body = MessagesRequestBody {
    model,
    messages,
    max_tokens,
    ..Default::default()
  };

  let response = client
    .create_a_message(request_body)
    .await?;

  let s = response.content.flatten_into_text()?;

  Ok(s.to_string())
}