mod client;
mod short_term_context;
mod dynamic_prompt;
mod run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run::run().await
}