use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Conversation {
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub user: bool,
    pub text: String,
}

impl Conversation {
    pub fn new() -> Conversation {
        Conversation {
            messages: Vec::new(),
        }
    }
}
