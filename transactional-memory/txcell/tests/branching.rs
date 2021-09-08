use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use txcell::tree::BinarySearchTree;

fn deterministic_rng() -> StdRng {
    let seed = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ]; // byte array
    StdRng::from_seed(seed)
}

// - create an empty binary tree
// - spawn N threads
// - in each thread:
//     - precompute random numbers
//     - start timer
//     - repeat the following transaction 100 times:
//         - insert one element into the tree or look up one element
//     - stop timer
// - sum timers, compute ops/sec
fn test_stm(
    num_threads: usize,
    percent_writes: f64,
    num_iterations: usize,
) -> Duration {
    // Create an empty binary tree
    let tree = Arc::new(BinarySearchTree::new(0));
    let mut total_time = Duration::new(0, 0);

    // spawn N threads
    let mut handles = vec![];
    for n in 0..num_threads {
        handles.push(thread::spawn({
            let tree_clone = Arc::clone(&tree);
            move || {
                // precompute random numbers
                let mut rng = deterministic_rng();
                let mut is_write = Vec::with_capacity(num_iterations);
                for _ in 0..num_iterations {
                    if rng.gen::<f64>() < percent_writes {
                        is_write.push(true);
                    } else {
                        is_write.push(false);
                    }
                }
                // start timer
                let now = Instant::now();
                // repeat the following transaction
                for i in 0..num_iterations {
                    let value = n * num_threads + i;
                    let mut _count = 0;
                    transaction {
                        if is_write[i] {
                            tree_clone.add(value);
                        } else {
                            if let Some(element) = tree_clone.find(value) {
                                let node = element.borrow();
                                _count += (*node).val;
                            }
                        }
                    }
                }
                // Stop timer
                now.elapsed()
            }
        }));
    }

    for handle in handles {
        total_time += handle.join().unwrap();
    }
    total_time
}

fn ops_per_second(duration: Duration, num_threads: usize, num_repeats: usize) -> f64 {
    let duration_ns = duration.as_nanos() as usize;
    let duration_sec = duration_ns as f64 / 10f64.powf(9.0);
    // number of operations = number of transactions.
    // m threads * num_repeats of tx per thread = number of transactions executed
    let ops_per_sec = (num_threads * num_repeats) as f64 / duration_sec;
    ops_per_sec
}

#[test]
fn branching() {
    // M cores, 1 thread per core
    let num_threads = vec![1];
    // Y% of accesses are writes
    let percent_writes = vec![0.0, 0.1, 0.25, 0.5, 0.75, 1.0];
    // Repeat this many times
    let num_repeats = 1;
    // output in CSV format
    println!("num_cores,num_accesses,percent_writes,stm_ops_per_sec,swym_ops_per_sec");
    for m in &num_threads {
        for y in &percent_writes {
            let duration_stm = test_stm(*m, *y, num_repeats);
            let ops_stm = ops_per_second(duration_stm, *m, num_repeats);

            println!("{},{},{},{}", m, y, ops_stm, 0);
        }
    }
}
