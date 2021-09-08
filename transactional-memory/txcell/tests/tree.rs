use std::sync::Arc;
use std::thread;
use txcell::tree::BinarySearchTree;

#[test]
fn tree() {
    const N: usize = 300;

    let tree = Arc::new(BinarySearchTree::new(0));

    // spawn N-1 threads to add N-1 elements to the tree (0 already in tree)
    let mut handles = vec![];
    for i in 1..N {
        handles.push(thread::spawn({
            let tree_clone = Arc::clone(&tree);
            move || {
                transaction {
                    tree_clone.add(i);
                }
            }
        }));
    }

    for handle in handles {
        let _ = handle.join().unwrap();
    }

    // check that all N elements exist in tree
    for i in 0..N {
        let found = tree.find(i);
        assert_eq!(true, found.is_some());
        let node = found.unwrap();
        let val = node.borrow().val;
        assert_eq!(val, i);
    }
}
