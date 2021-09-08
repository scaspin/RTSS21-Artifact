//! Tests that look at how control flow affects the conflict sets.
use std::sync::{Arc, Mutex};
use std::{thread, time};
use txcell::TxPtr;

// A simple branching example with one thread
// Will always pass
#[test]
fn branch_one() {
    let a = Arc::new(TxPtr::new(2));
    let a_saved = a.clone();
    let t1 = thread::spawn(move || {
        transaction {
            if rand::random() {
                let t = a.borrow_mut();
                thread::sleep(time::Duration::from_millis(10));
                *t += 5;
            } else {
                let t = a.borrow_mut();
                thread::sleep(time::Duration::from_millis(10));
                *t += 3;
            }
        }
    });
    let _ = t1.join();
    assert!((*a_saved.borrow() == 7) | (*a_saved.borrow() == 5));
}

// A branching example with two threads
// Includes a 1ms delay so that interesting things happen
// Will always pass
#[test]
fn branch_two() {
    let a = Arc::new(TxPtr::new(0));
    let a_clone = a.clone();
    let a_saved = a.clone();
    let t1 = thread::spawn(move || {
        transaction {
            if rand::random() {
                let t = a.borrow_mut();
                thread::sleep(time::Duration::from_millis(1));
                *t += 5;
            } else {
                let t = a.borrow_mut();
                thread::sleep(time::Duration::from_millis(1));
                *t += 3;
            }
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            if rand::random() {
                let t = a_clone.borrow_mut();
                thread::sleep(time::Duration::from_millis(1));
                *t += 1;
            } else {
                let t = a_clone.borrow_mut();
                thread::sleep(time::Duration::from_millis(1));
                *t += 3;
            }
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    assert!((*a_saved.borrow() == 4) | (*a_saved.borrow() == 6) | (*a_saved.borrow() == 8));
}

/// Use the same TxPtr in every iteration, so each transaction should conflict.
#[test]
fn loop_shared() {
    let rc = TxPtr::new(1);
    let a = Arc::new(rc);
    let a_saved = a.clone();

    let n = 300;

    let mut threads = vec![];

    for _ in 1..n {
        let a_clone = a.clone();
        threads.push(thread::spawn(move || {
            transaction {
                let rc_ref = a_clone.borrow_mut();
                *rc_ref += 1;
            }
        }));
    }

    // wait for all threads to finish
    for t in threads {
        let _ = t.join();
    }

    // NB: "developers, don't be stupid and write transactions in single-threaded code."
    // BCW: "there's a way to refactor to avoid this conflict."
    assert_eq!(*a_saved.borrow(), n);
}

/// Add 1 or 0 to both a a TxPtr in a transaction and a Mutex randomly and assert
/// that the results are the same at the end.
#[test]
fn compare_to_mutex() {
    let rc = TxPtr::new(0);
    let a = Arc::new(rc);
    let a_saved = a.clone();

    let mu = Mutex::new(0);
    let b = Arc::new(mu);
    let b_saved = b.clone();

    let n = 300;

    let mut threads = vec![];
    for _ in 1..n {
        let a_clone = a.clone();
        let b_clone = b.clone();
        threads.push(thread::spawn(move || {
            let rand = rand::random::<bool>();
            // STM version
            transaction {
                let a_ref = a_clone.borrow_mut();
                if rand {
                    *a_ref += 1;
                }
            }
            // lock version
            let mut b_ref = b_clone.lock().unwrap();
            if rand {
                *b_ref += 1;
            }
        }));
    }

    for t in threads {
        let _ = t.join();
    }

    assert_eq!(*a_saved.borrow(), *b_saved.lock().unwrap());
}

#[test]
fn test_indirect() {
    foo();
    bar();
}

fn foo() {
    let a = Arc::new(TxPtr::new(2));
    let a_clone = a.clone();
    let a_saved = a.clone();
    baz(a);
    transaction {
        let r = a_clone.borrow_mut();
        *r += 2;
    }
    assert_eq!(*a_saved.borrow(), 5);
}

fn bar() {
    let b = Arc::new(TxPtr::new(1));
    let b_clone = b.clone();
    let b_saved = b.clone();
    baz(b);
    transaction {
        let r = b_clone.borrow_mut();
        *r += 2;
    }
    assert_eq!(*b_saved.borrow(), 4);
}

fn baz(c: Arc<TxPtr<i32>>) {
    transaction {
        let r = c.borrow_mut();
        *r += 1;
    }
}
