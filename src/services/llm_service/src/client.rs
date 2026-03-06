use serde::{Deserialize, Serialize};
use reqwest::{self, Client};
use serde::ser::StdError;

use crate::client;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ModelResponse {
    #[serde(rename = "model")]
    pub model_name: String,

    #[serde(rename = "created_at")]
    pub created_at: String,

    pub message: Messages,

    #[serde(rename = "done")]
    pub done: bool,

    #[serde(rename = "done_reason")]
    pub done_reason: String,

    #[serde(rename = "total_duration")]
    pub total_duration: i64,

    #[serde(rename = "load_duration")]
    pub load_duration: i64,

    #[serde(rename = "prompt_eval_count")]
    pub prompt_eval_count: i64,

    #[serde(rename = "prompt_eval_duration")]
    pub prompt_eval_duration: i64,
    
    #[serde(rename = "eval_count")]
    pub eval_count: i64,

    #[serde(rename = "eval_duration")]
    pub eval_duration: i64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Messages>,
    pub stream: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Messages {
    #[serde(rename = "role")]
    pub role: String,
    
    #[serde(rename = "content")]
    pub content: String
}

impl ModelResponse {
    pub async fn llm(messages: Vec<Messages>) -> Result<client::ModelResponse, Box<dyn StdError + 'static>> {
        let url = "http://localhost:11434/api/chat";
        let client = Client::new();

        let request = ChatRequest {
            model: "codegemma:7b-instruct".to_string(),
            messages,
            stream: false,
        };

        let response = client.post(url)
            .json(&request)
            .send()
            .await?
            .json::<Self>()
            .await?;

        Ok(response)
    }
}