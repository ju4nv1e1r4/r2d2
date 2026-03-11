use std::sync::atomic::{AtomicU64, Ordering};
use crate::client::Messages;

// Contador global monotônico. Garante chaves únicas e estritamente
// crescentes sem depender de resolução de clock.
static INSERT_COUNTER: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// TreapNode
// ---------------------------------------------------------------------------

struct TreapNode {
    // Chave BST: ordem de inserção. Determina a posição na árvore e,
    // consequentemente, a ordem do traversal in-order.
    insert_order: u64,

    // Prioridade heap: aleatória. Mantém a árvore balanceada.
    // Um nó pai sempre tem priority > que seus filhos.
    priority: u32,

    message: Messages,
    left:  Option<Box<TreapNode>>,
    right: Option<Box<TreapNode>>,
}

impl TreapNode {
    fn new(message: Messages) -> Box<Self> {
        Box::new(TreapNode {
            insert_order: INSERT_COUNTER.fetch_add(1, Ordering::Relaxed),
            priority: rand_u32(),
            message,
            left: None,
            right: None,
        })
    }
}

// ---------------------------------------------------------------------------
// Rotações
//
//   split_right (rotate left):
//
//       N                 R
//      / \               / \
//     L   R    →        N   RR
//        / \           / \
//       RL  RR        L   RL
//
//   split_left (rotate right):
//
//       N                L
//      / \               / \
//     L   R    →        LL   N
//    / \                    / \
//   LL  LR                 LR  R
// ---------------------------------------------------------------------------

/// Rotaciona para a esquerda: eleva o filho direito.
/// Usado quando right.priority > node.priority (viola heap).
fn rotate_left(mut node: Box<TreapNode>) -> Box<TreapNode> {
    let mut right = node.right.take().expect("rotate_left: right is None");
    node.right = right.left.take();
    right.left = Some(node);
    right
}

/// Rotaciona para a direita: eleva o filho esquerdo.
/// Usado quando left.priority > node.priority (viola heap).
fn rotate_right(mut node: Box<TreapNode>) -> Box<TreapNode> {
    let mut left = node.left.take().expect("rotate_right: left is None");
    node.left = left.right.take();
    left.right = Some(node);
    left
}

// ---------------------------------------------------------------------------
// Operações sobre Option<Box<TreapNode>>
// ---------------------------------------------------------------------------

/// Insere um novo nó na subárvore e corrige invariantes via rotações.
fn insert(subtree: Option<Box<TreapNode>>, new_node: Box<TreapNode>) -> Box<TreapNode> {
    match subtree {
        None => new_node,

        Some(mut node) => {
            if new_node.insert_order < node.insert_order {
                // Vai para a esquerda (BST)
                node.left = Some(insert(node.left.take(), new_node));

                // Corrige heap: se o filho esquerdo tem priority maior, rotaciona
                if node.left.as_ref().unwrap().priority > node.priority {
                    node = rotate_right(node);
                }
            } else {
                // Vai para a direita (BST)
                node.right = Some(insert(node.right.take(), new_node));

                // Corrige heap: se o filho direito tem priority maior, rotaciona
                if node.right.as_ref().unwrap().priority > node.priority {
                    node = rotate_left(node);
                }
            }

            node
        }
    }
}

/// Remove o nó com o menor insert_order da subárvore (o mais antigo).
/// Retorna (subárvore resultante, nó removido).
fn remove_min(node: Box<TreapNode>) -> (Option<Box<TreapNode>>, Box<TreapNode>) {
    match node.left {
        None => {
            // Este nó é o mínimo: não tem filho esquerdo.
            // O filho direito sobe no lugar dele.
            let right = node.right;
            let mut removed = node;
            removed.right = None;
            (right, removed)
        }
        Some(ref left) => {
            // Desce à esquerda procurando o mínimo.
            let (new_left, removed) = remove_min( left);
            let mut node = node;
            node.left = new_left;
            (Some(node), removed)
        }
    }
}

/// Traversal in-order: visita nós em ordem crescente de insert_order
/// (= ordem cronológica de inserção).
fn in_order<'a>(node: &'a Option<Box<TreapNode>>, out: &mut Vec<&'a Messages>) {
    if let Some(n) = node {
        in_order(&n.left, out);
        out.push(&n.message);
        in_order(&n.right, out);
    }
}

/// Retorna a altura da subárvore. Útil para debug/inspeção.
#[allow(dead_code)]
fn height(node: &Option<Box<TreapNode>>) -> usize {
    match node {
        None => 0,
        Some(n) => 1 + height(&n.left).max(height(&n.right)),
    }
}

// ---------------------------------------------------------------------------
// ShortTermMemory (interface pública)
// ---------------------------------------------------------------------------

pub struct ShortTermMemory {
    root: Option<Box<TreapNode>>,
    max_nodes: usize,
    current_count: usize,
}

impl ShortTermMemory {
    pub fn new(max_nodes: usize) -> Self {
        ShortTermMemory {
            root: None,
            max_nodes,
            current_count: 0,
        }
    }

    /// Insere uma mensagem.
    /// Se current_count == max_nodes, remove o nó mais antigo antes de inserir.
    /// Sliding window: nunca perde o contexto inteiro de uma vez.
    /// Complexidade: O(log n) esperado.
    pub fn store(&mut self, message: Messages) {
        if self.current_count >= self.max_nodes {
            self.remove_oldest();
        }

        let new_node = TreapNode::new(message);
        let root = self.root.take();
        self.root = Some(insert(root, new_node));
        self.current_count += 1;
    }

    /// Remove o nó mais antigo (menor insert_order).
    fn remove_oldest(&mut self) {
        if let Some(root) = self.root.take() {
            let (new_root, _removed) = remove_min(&root);
            self.root = new_root;
            self.current_count -= 1;
        }
    }

    /// Retorna o histórico em ordem cronológica (mais antigo → mais recente).
    /// Complexidade: O(n).
    pub fn get_ordered_history(&self) -> Vec<Messages> {
        let mut refs: Vec<&Messages> = Vec::with_capacity(self.current_count);
        in_order(&self.root, &mut refs);

        refs.iter()
            .map(|m| Messages {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.current_count
    }

    pub fn is_empty(&self) -> bool {
        self.current_count == 0
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.current_count = 0;
    }

    /// Retorna a altura atual da árvore. Deve ser próxima de log2(n).
    /// Use para validar que o balanceamento está funcionando.
    #[allow(dead_code)]
    pub fn tree_height(&self) -> usize {
        height(&self.root)
    }
}

// ---------------------------------------------------------------------------
// PRNG — xorshift32
//
// Sem dependência externa. Seed via pointer de stack (endereço de variável
// local) XOR com um contador global — suficiente para distribuição aleatória
// das prioridades do treap.
//
// NÃO é criptograficamente seguro. Não precisa ser.
// ---------------------------------------------------------------------------

fn rand_u32() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};

    // Seed inicial: endereço de uma variável local (varia por thread/call)
    // misturado com um contador para evitar repetição.
    static SEED: AtomicU32 = AtomicU32::new(0);

    let stack_addr = {
        let local = 0u32;
        &local as *const u32 as u32
    };

    let prev = SEED.load(Ordering::Relaxed);
    let new_seed = if prev == 0 {
        stack_addr.wrapping_add(2_463_534_242) // constante Knuth
    } else {
        prev
    };

    // xorshift32
    let mut x = new_seed ^ stack_addr;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;

    SEED.store(x, Ordering::Relaxed);
    x
}