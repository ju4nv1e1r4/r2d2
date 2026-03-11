use std::error::Error;
use std::io::{self, Write};
use crate::client::{Messages, ModelResponse};
use crate::dynamic_prompt::DynamicPromptManager;
use crate::short_term_context_deq::ShortTermMemory;
use crate::utils::{measure_execution, PerformanceAnalyzer};

pub async fn run() -> Result<(), Box<dyn Error>> {
    let mut memory = ShortTermMemory::new(20);
    
    let mut prompt_manager = DynamicPromptManager::new(
        "Você é um Engenheiro de Software Sênior especializado em AI, Machine Learning e MLOps."
    );
    prompt_manager.set_task("Explicar conceitos de desenvolvimento e auxiliar no código.");
    
    let mut performance_analyzer = PerformanceAnalyzer::new();

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

        // Mede o tempo de construção do prompt (síncrono)
        let (full_prompt, prompt_duration) = measure_execution(|| {
            prompt_manager.build_contextual_prompt(user_query, &memory)
        });

        println!("--- Pensando... (Contexto montado em {:.2?}) ---", prompt_duration);

        match ModelResponse::generate(full_prompt).await {
            Ok(response) => {
                let assistant_answer = response.message.content.clone();

                println!("\nIA: {}", assistant_answer);

                // Captura o tamanho da memória antes de adicionar as novas mensagens para o relatório
                let memory_count_for_report = memory.get_ordered_history().len();

                memory.store(Messages {
                    role: "user".to_string(),
                    content: user_query.to_string(),
                });

                memory.store(Messages {
                    role: "assistant".to_string(),
                    content: assistant_answer,
                });

                println!("\n[Memória BST: {} mensagens]", memory.get_ordered_history().len());

                performance_analyzer.add_report(&response, memory_count_for_report);
                performance_analyzer.print_last_report();
                performance_analyzer.analyze_bottlenecks();
            }
            Err(e) => eprintln!("Erro na comunicação com Ollama: {}", e),
        }
    }

    println!("Até logo!");
    Ok(())
}
