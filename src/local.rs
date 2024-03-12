use llama_cpp_rs::LLama;
use llama_cpp_rs::options::{ModelOptions, PredictOptions};

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
  local_llm.predict(prompt.into(), options)
}