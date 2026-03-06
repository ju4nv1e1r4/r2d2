use crate::short_term_context::ShortTermMemory;
use crate::client::Messages;

pub struct DynamicPromptManager {
    pub system_base: String,
    pub current_task: Option<String>,
}

impl DynamicPromptManager {
    pub fn new(role_description: &str) -> Self {
        Self {
            system_base: role_description.to_string(),
            current_task: None,
        }
    }

    pub fn set_task(&mut self, task_description: &str) {
        self.current_task = Some(task_description.to_string());
    }

    pub fn build_contextual_prompt(&self, user_query: &str, memory: &ShortTermMemory) -> Vec<Messages> {
        let mut full_messages = Vec::new();

        let mut system_content = format!("## PERSONA\n{}\n", self.system_base);
        if let Some(task) = &self.current_task {
            system_content.push_str(&format!("## TAREFA ATUAL \n{}\n", task));
        }

        full_messages.push(Messages {
            role: "system".to_string(),
            content: system_content,
        });

        let history = memory.get_ordered_history();
        for msg in history {
            full_messages.push(msg);
        }

        full_messages.push(Messages {
            role: "user".to_string(),
            content: user_query.to_string(),
        });

        full_messages
    }
}