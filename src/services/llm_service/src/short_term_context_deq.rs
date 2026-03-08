use std::collections::VecDeque;
use crate::client::Messages;

pub struct ShortTermMemory {
    history: VecDeque<Messages>,
    max_nodes: usize,
}

#[allow(dead_code)]
impl ShortTermMemory {
    pub fn new(max_nodes: usize) -> Self {
        ShortTermMemory {
            history: VecDeque::with_capacity(max_nodes),
            max_nodes,
        }
    }

    pub fn store(&mut self, message: Messages) {
        if self.history.len() >= self.max_nodes {
            self.history.pop_front();
        }
        self.history.push_back(message);
    }

    pub fn get_ordered_history(&self) -> Vec<Messages> {
        self.history
            .iter()
            .map(|m| Messages {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }
}
