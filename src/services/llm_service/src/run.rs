use std::error::Error;
use crate::{client::{Messages, ModelResponse}, dynamic_prompt::DynamicPromptManager, short_term_context::{MemorySortStrategy, ShortTermMemory}};

pub async fn run() -> Result<(), Box<dyn Error>> {
    let mut memory = ShortTermMemory::new(20, Some(MemorySortStrategy::TimeStamp));
    
    let mut prompt_manager = DynamicPromptManager::new(
        "Você é um Engenheiro de Software Sênior especializado em AI, Machine Learning e MLOps."
    );
    prompt_manager.set_task("Explicar conceitos fundamentais de Rust para um desenvolvedor pleno.");

    let user_query = "Vec<String> in rust, whats that?";

    let full_prompt = prompt_manager.build_contextual_prompt(user_query, &memory);

    println!("--- Enviando Prompt com Contexto ---");

    match ModelResponse::llm(full_prompt).await {
        Ok(response) => {
            let assistant_answer = response.message.content.clone();
            
            println!("IA: {}", assistant_answer);

            memory.store(Messages {
                role: "user".to_string(),
                content: user_query.to_string(),
            }, None);

            memory.store(Messages {
                role: "assistant".to_string(),
                content: assistant_answer,
            }, Some(0.8));

            println!("\n[Memória atualizada na BST. Total de nós: {}]", memory.get_ordered_history().len());
        }
        Err(e) => eprintln!("Erro na comunicação com Ollama: {}", e),
    }

    Ok(())
}