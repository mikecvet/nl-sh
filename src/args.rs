pub enum ModelType {
  GPT4,
  GPT35,
  Claude,
  Local(String),
}

pub struct Args {
  /// Represents the model selected by the user based on command-line arguments
  pub model_type: ModelType,

  /// Indicates whether any command history should be written to the user's native shell history file
  pub stateless: bool
}

impl Args {
  pub fn new (matches: &clap::ArgMatches) -> Self 
  {
    let gpt4 = matches.get_one::<bool>("gpt4").unwrap_or(&true); // Default to true
    let gpt35 = matches.get_one::<bool>("gpt35").unwrap_or(&false);
    let claude = matches.get_one::<bool>("claude").unwrap_or(&false);
    let local_opt = matches.get_one::<String>("local").cloned();

    let model_type = if let Some(path) = local_opt {
        ModelType::Local(path)
    } else if *gpt35 {
        ModelType::GPT35
    } else if *claude {
        ModelType::Claude
    } else if *gpt4 {
        ModelType::GPT4
    } else {
        panic!("No model specified");
    };

    let stateless = matches.get_one::<bool>("stateless").map(|&b| b).unwrap_or(false);

    Args { model_type, stateless }
  }
}
