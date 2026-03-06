use std::{env::current_exe, time::SystemTime};
use serde::de::value::BoolDeserializer;
use tokio::io::unix::AsyncFdTryNewError;

use crate::client::Messages;

#[derive(Debug, Clone, Copy)]
pub enum MemorySortStrategy {
    TimeStamp,
    Score
}

#[derive(Debug)]
pub struct MemoryNode {
    pub timestamp: u64,
    pub score: f32,
    pub message: Messages,
    pub left: Option<Box<MemoryNode>>,
    pub right: Option<Box<MemoryNode>>,
}

impl MemoryNode {
    pub fn new(message: Messages, score: Option<f32>) -> Self {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        MemoryNode { 
            timestamp: now, 
            score: score.unwrap_or(0.0), 
            message: message, 
            left: None, 
            right: None
        }
    }

    pub fn get_key(&self, strategy: MemorySortStrategy) -> f64 {
        match strategy {
            MemorySortStrategy::TimeStamp => self.timestamp as f64,
            MemorySortStrategy::Score => self.score as f64,
        }
    }
}

pub struct ShortTermMemory {
    root: Option<Box<MemoryNode>>,
    strategy: MemorySortStrategy,
    max_nodes: usize,
    current_count: usize,
}

impl ShortTermMemory {
    pub fn new(max_nodes: usize, strategy: Option<MemorySortStrategy>) -> Self {
        ShortTermMemory { 
            root: None,
            strategy: strategy.unwrap_or(MemorySortStrategy::TimeStamp),
            max_nodes: max_nodes,
            current_count: 0,
        }
    }

    pub fn score(&mut self, message: Messages, score: Option<f32>) {
        if self.current_count >= self.max_nodes {
            self.clear();
        }

        let new_node = Box::new(MemoryNode::new(message, score));
        let strategy = self.strategy;

        if self.root.is_none() {
            self.root = Some(new_node);
        } else {
            Self::insert_recursive(self.root.as_mut().unwrap(), new_node, strategy);
        }

        self.current_count += 1
    }

    pub fn insert_recursive(current: &mut Box<MemoryNode>, new_node: Box<MemoryNode>, strategy: MemorySortStrategy) {
        let new_key = new_node.get_key(strategy);
        let current_key = current.get_key(strategy);

        if new_key < current_key {
            if let Some(ref mut left) = current.left {
                Self::insert_recursive(left, new_node, strategy);
            } else {
                current.left = Some(new_node);
            }
        } else {
            if let Some(ref mut right) = current.right {
                Self::insert_recursive(right, new_node, strategy);
            } else {
                current.right = Some(new_node);
            }
        }
    }

    pub fn get_ordered_history(&self) -> Vec<Messages>{
        let mut history = Vec::new();
        Self::in_order_traversal(&self.root, &mut history);

        history
    }

    fn in_order_traversal(node: &Option<Box<MemoryNode>>, history: &mut Vec<Messages>) {
        if let Some(n) = node {
            Self::in_order_traversal(&n.left, history);
            history.push(Messages {
                role: n.message.role.clone(),
                content: n.message.content.clone()
            });

            Self::in_order_traversal(&n.right, history);
        }
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.current_count = 0;
    }
}
