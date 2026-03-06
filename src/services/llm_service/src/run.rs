use std::error::Error;
use std::io::{self, Write};
use crate::client::{Messages, ModelResponse};
use crate::dynamic_prompt::DynamicPromptManager;
use crate::short_term_context::{MemorySortStrategy, ShortTermMemory};

pub async fn run() -> Result<(), Box<dyn Error>> {
    // A memória é instanciada FORA do loop para persistir entre as perguntas
    let mut memory = ShortTermMemory::new(20, Some(MemorySortStrategy::TimeStamp));
    
    let mut prompt_manager = DynamicPromptManager::new(
        "Você é um Engenheiro de Software Sênior especializado em AI, Machine Learning e MLOps."
    );
    prompt_manager.set_task("Explicar conceitos de desenvolvimento e auxiliar no código.");

    println!("--- Agente CLI Iniciado (digite 'sair' para encerrar) ---");

    loop {
        print!("\nVocê: ");
        io::stdout().flush()?;

        let mut user_query = String::new();
        io::stdin().read_line(&mut user_query)?;
        let user_query = user_query.trim();

        if user_query.is_empty() {
            continue;
        }
        if user_query.to_lowercase() == "sair" {
            break;
        }

        let full_prompt = prompt_manager.build_contextual_prompt(user_query, &memory);

        println!("--- Pensando... ---");

        match ModelResponse::llm(full_prompt).await {
            Ok(response) => {
                let assistant_answer = response.message.content.clone();

                println!("\nIA: {}", assistant_answer);

                memory.store(Messages {
                    role: "user".to_string(),
                    content: user_query.to_string(),
                }, None);

                memory.store(Messages {
                    role: "assistant".to_string(),
                    content: assistant_answer,
                }, Some(0.8));

                println!("\n[Memória BST: {} mensagens]", memory.get_ordered_history().len());
            }
            Err(e) => eprintln!("Erro na comunicação com Ollama: {}", e),
        }
    }

    println!("Até logo!");
    Ok(())
}
