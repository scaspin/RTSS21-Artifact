use std::sync::Arc;
use std::thread;
use txcell::TxPtr;

#[test]
fn array() {
    const N: usize = 3;
    const X: usize = 300;

    let mut cells = Vec::new();
    for _ in 0..N {
        cells.push(Arc::new(TxPtr::new(0)));
    }

    let mut threads = vec![];
    for i in 0..N {
        for _ in 0..X {
            let arc_clone = Arc::clone(&cells[i]);
            threads.push(thread::spawn(move || {
                transaction {
                    let rc_ref = arc_clone.borrow_mut();
                    *rc_ref += 1;
                }
            }));
        }
    }

    // wait for all threads to finish
    for t in threads {
        let _ = t.join();
    }

    let mut sum = 0;
    for cell in cells {
        sum += *cell.borrow();
    }
    assert_eq!(sum, X * N);
}
