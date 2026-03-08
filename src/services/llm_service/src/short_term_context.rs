use std::time::SystemTime;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::client::Messages;

// Contador monotônico global: chave de desempate quando dois nós
// chegam no mesmo nanosegundo (raro, mas possível).
static INSERT_COUNTER: AtomicU64 = AtomicU64::new(0);

#[allow(dead_code)]
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
        let nanos = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;

        let timestamp = nanos ^ INSERT_COUNTER.fetch_add(1, Ordering::Relaxed);

        MemoryNode {
            timestamp,
            score: score.unwrap_or(0.0),
            message,
            left: None,
            right: None,
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

#[allow(dead_code)]
impl ShortTermMemory {
    pub fn new(max_nodes: usize, strategy: Option<MemorySortStrategy>) -> Self {
        ShortTermMemory { 
            root: None,
            strategy: strategy.unwrap_or(MemorySortStrategy::TimeStamp),
            max_nodes: max_nodes,
            current_count: 0,
        }
    }

    pub fn store(&mut self, message: Messages, score: Option<f32>) {
        if self.current_count >= self.max_nodes {
            self.remove_oldest();
        }

        let new_node = Box::new(MemoryNode::new(message, score));
        let strategy = self.strategy;

        if self.root.is_none() {
            self.root = Some(new_node);
        } else {
            Self::insert_recursive(self.root.as_mut().unwrap(), new_node, strategy);
        }

        self.current_count += 1;
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

    fn remove_oldest(&mut self) {
        if self.root.is_none() {
            return;
        }

        self.root = Self::remove_min(self.root.take());
        self.current_count -= 1;
    }

    fn remove_min(node: Option<Box<MemoryNode>>) -> Option<Box<MemoryNode>> {
        match node {
            None => None,
            Some(mut n) => {
                if n.left.is_none() {
                    // Este é o mínimo: substitui pela subárvore direita.
                    n.right.take()
                } else {
                    n.left = Self::remove_min(n.left.take());
                    Some(n)
                }
            }
        }
    }

}
