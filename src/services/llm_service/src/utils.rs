use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::client::ModelResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceMetrics {
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub tokens_memory: i32,
    pub total_duration_ms: f64,
    pub prompt_eval_ms: f64,
    pub eval_ms: f64,
    pub tokens_per_second: f64,
}

pub struct PerformanceAnalyzer {
    pub reports: Vec<PerformanceMetrics>,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self { reports: Vec::new() }
    }

    pub fn add_report(&mut self, response: &ModelResponse, memory_count: usize) {
        let total_ms = response.total_duration as f64 / 1_000_000.0;
        let eval_ms = response.eval_duration as f64 / 1_000_000.0;

        let tps = if eval_ms > 0.0 {
            (response.eval_count as f64 / eval_ms) * 1000.0
        } else {
            0.0
        };

        let metrics = PerformanceMetrics {
            tokens_input: response.prompt_eval_count,
            tokens_output: response.eval_count,
            tokens_memory: (memory_count * 15) as i32,
            total_duration_ms: total_ms,
            prompt_eval_ms: response.prompt_eval_duration as f64 / 1_000_000.0,
            eval_ms,
            tokens_per_second: tps,
        };

        self.reports.push(metrics);
    }

    pub fn print_last_report(&self) {
        if let Some(m) = self.reports.last() {
            println!("\nRELATÓRIO DE PERFORMANCE");
            println!("Tokens: {} in / {} out (Memória: ~{} tokens)", m.tokens_input, m.tokens_output, m.tokens_memory);
            println!("Latência Total: {:.2}ms", m.total_duration_ms);
            println!("Prompt Processing: {:.2}ms", m.prompt_eval_ms);
            println!("Geração (Eval): {:.2}ms", m.eval_ms);
            println!("Velocidade: {:.2} tokens/s", m.tokens_per_second);
            println!("------------------------------------\n");
        }
    }

    pub fn analyze_bottlenecks(&self) {
        if let Some(m) = self.reports.last() {
            if m.prompt_eval_ms > m.eval_ms * 0.5 {
                println!("[CRÍTICA] O processamento do prompt está a ocupar >50% do tempo. A memória BST pode estar a enviar demasiados tokens de contexto.");
            }
            if m.tokens_per_second < 10.0 {
                println!("[CRÍTICA] Velocidade de geração baixa ({:.2} t/s). Possível falta de recursos (GPU/RAM) ou modelo muito pesado.", m.tokens_per_second);
            }
        }
    }
}

pub fn measure_execution<T, F: FnOnce() -> T>(f: F) -> (T, Duration) {
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}
