mod client;
mod short_term_context;
use client::ModelResponse;

#[tokio::main]
async fn main() {
    let user_input = String::from("Vec<String> in rust, whats that?");

    let result = ModelResponse::llm(user_input);
    let response = result.await.unwrap();
    println!("{:#?}", response); 
}
