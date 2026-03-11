mod client;
mod short_term_context_deq;
mod dynamic_prompt;
mod run;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run::run().await
}
