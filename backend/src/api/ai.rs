use serde::{Deserialize, Serialize};
use reqwest::{Client, ClientBuilder};
use std::env;
use std::time::Duration;

const API_URL: &str = "https://qianfan.baidubce.com/v2/chat/completions";
const MAX_RETRIES: u32 = 3;
const TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

pub async fn send_chat_request(messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()?;
    // let api_key = env::var("ERNIE_API_KEY").expect("ERNIE_API_KEY must be set");
    let api_key = "bce-v3/ALTAK-Wk3XBWa75x1cjYHkDrcHn/13e33a22e2b698831729d17dfd5370261ae227b7";

    let request = ChatRequest {
        model: "ernie-3.5-8k".to_string(),
        messages,
    };

    let mut last_error = None;
    for retry in 0..MAX_RETRIES {
        if retry > 0 {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        match make_request(&client, &api_key, &request).await {
            Ok(content) => return Ok(content),
            Err(e) => {
                last_error = Some(e);
                continue;
            }
        }
    }

    Err(last_error.unwrap())
}

async fn make_request(
    client: &Client,
    api_key: &str,
    request: &ChatRequest,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = client
        .post(API_URL)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(request)
        .send()
        .await?
        .json::<ChatResponse>()
        .await?;
    
    Ok(response.choices[0].message.content.clone())
} 