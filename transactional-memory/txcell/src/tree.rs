use crate::TxPtr;
use std::cmp::Ordering;
use std::sync::Arc;

type BareTree<T> = Arc<TxPtr<Node<T>>>;
type Tree<T> = Option<BareTree<T>>;

#[derive(Debug)]
pub struct Node<T: Ord> {
    pub val: T,
    left: Tree<T>,
    right: Tree<T>,
}

#[derive(Clone)]
pub struct BinarySearchTree<T: Ord> {
    root: Tree<T>,
}

impl<T: Ord> Node<T> {
    fn new(val: T) -> Tree<T> {
        Some(Arc::new(TxPtr::new(Node {
            val: val,
            left: None,
            right: None,
        })))
    }
}

impl<T: Ord + Clone + std::fmt::Debug> BinarySearchTree<T> {
    pub fn new(val: T) -> BinarySearchTree<T> {
        BinarySearchTree {
            root: Node::new(val),
        }
    }

    pub fn add(&self, val: T) {
        if let Some(root) = &self.root {
            let root_clone = Arc::clone(&root);
            self.add_r(Some(root_clone), val);
        }
    }

    fn add_r(&self, node: Tree<T>, val: T) -> (Tree<T>, BareTree<T>) {
        if let Some(n) = node {
            let new: BareTree<T>;
            let current_val = n.borrow().val.clone();
            if &current_val <= &val {
                let left = n.borrow().left.clone();
                let new_tree = self.add_r(left, val);
                new = new_tree.1;
                let new_tree = new_tree.0.unwrap();
                n.borrow_mut().left = Some(new_tree);
            } else {
                let right = n.borrow().right.clone();
                let new_tree = self.add_r(right, val);
                new = new_tree.1;
                let new_tree = new_tree.0.unwrap();
                n.borrow_mut().right = Some(new_tree);
            }
            (Some(n), new)
        } else {
            let new = Node::new(val);
            (new.clone(), new.unwrap())
        }
    }

    pub fn find(&self, val: T) -> Tree<T> {
        self.find_r(&self.root, &val)
    }

    fn find_r(&self, node: &Tree<T>, val: &T) -> Tree<T> {
        match node {
            Some(n) => {
                let n_ref = n.borrow();
                match n_ref.val.cmp(&val) {
                    Ordering::Less => self.find_r(&n_ref.left, val),
                    Ordering::Equal => Some(Arc::clone(n)),
                    Ordering::Greater => self.find_r(&n_ref.right, val),
                }
            }
            _ => None,
        }
    }

    pub fn walk(&self, callback: impl Fn(&T) -> ()) {
        self.walk_in_order(&self.root, &callback);
    }

    fn walk_in_order(&self, node: &Tree<T>, callback: &impl Fn(&T) -> ()) {
        if let Some(n) = node {
            let n = n.borrow();

            self.walk_in_order(&n.left, callback);
            callback(&n.val);
            self.walk_in_order(&n.right, callback);
        }
    }
}
