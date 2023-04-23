use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Prompt {
    text: String,
    source: Option<String>,
}

pub trait Prompter {
    type Iter: Iterator<Item = Prompt>;
    fn prompts(&self) -> Self::Iter;
}
