//! Tests that look at how function calls and threads affect the allocation sets
//! for the transactions.
use std::sync::Arc;
use std::{thread, time};
use txcell::TxPtr;
use std::ops::{AddAssign, SubAssign};

/// Simple example: one TxPtr, one thread, one transaction.
#[test]
fn one_thread() {
    let rc = TxPtr::new(1);
    let a = Arc::new(rc);
    let a_saved = a.clone();

    let t1 = thread::spawn(move || {
        transaction {
            let rc_ref = a.borrow_mut();
            *rc_ref += 1;
        }
    });
    let _ = t1.join();
    assert_eq!(*a_saved.borrow(), 2);
}

// Two possible allocation sites and a thread.
#[test]
fn alloc_set_and_threads() {
    let rc;
    if rand::random() {
        rc = TxPtr::new(0);
    } else {
        rc = TxPtr::new(1);
    }
    let a = Arc::new(rc);
    let a_saved = a.clone();
    let t1 = thread::spawn(move || {
        transaction {
            let t = a.borrow_mut();
            *t += 1;
        }
    });
    let _ = t1.join();
    assert!(*a_saved.borrow() == 1 || *a_saved.borrow() == 2);
}

fn increment_with_tx<T: AddAssign<T> + From<i32>>(a: Arc<TxPtr<T>>) {
    transaction {
        let rc_ref = a.borrow_mut();
        *rc_ref += T::from(1);
    }
}

fn decrement_with_tx<T: SubAssign<T> + From<i32>>(a: Arc<TxPtr<T>>) {
    transaction {
        let rc_ref = a.borrow_mut();
        *rc_ref -= T::from(1);
    }
}

/// Thread closure calls a function that contains a transaction
#[test]
fn function_call() {
    let rc = TxPtr::new(1);
    let a = Arc::new(rc);
    let a_saved = a.clone();

    let t1 = thread::spawn(|| increment_with_tx(a));
    let _ = t1.join();
    assert_eq!(*a_saved.borrow(), 2);
}

/// Two threads each call the same function that contains a transaction
/// TODO: A and B conflict right now. Need further analysis to deal with this.
#[test]
fn function_call_twice() {
    let a = Arc::new(TxPtr::new(0));
    let a_saved = a.clone();
    let b = Arc::new(TxPtr::new(0.7));
    let b_saved = b.clone();

    let t1 = thread::spawn(|| increment_with_tx(a));
    let t2 = thread::spawn(|| increment_with_tx(b));
    let _ = t1.join();
    let _ = t2.join();
    assert_eq!(*a_saved.borrow(), 1);
    assert_eq!(*b_saved.borrow(), 1.7);
}

/// Two threads each call different functions that contain a transaction.
/// TODO: make sure that the two transactions conflict.
#[test]
fn different_transactions() {
    let a = Arc::new(TxPtr::new(0));
    let a_clone = a.clone();
    let a_saved = a.clone();

    let t1 = thread::spawn(|| increment_with_tx(a));
    let t2 = thread::spawn(|| decrement_with_tx(a_clone));
    let _ = t1.join();
    let _ = t2.join();
    assert_eq!(*a_saved.borrow(), 0);
}

fn increment(a: Arc<TxPtr<i32>>) {
    let rc_ref = a.borrow_mut();
    *rc_ref += 1;
}

/// Two threads use different Arcs
/// T1: A
/// T2: B
#[test]
fn disjoint_sets() {
    let a = Arc::new(TxPtr::new(0));
    let a_saved = a.clone();

    let b = Arc::new(TxPtr::new(0));
    let b_saved = b.clone();

    let t1 = thread::spawn(move || {
        transaction {
            increment(a);
        }
    });

    let t2 = thread::spawn(move || {
        transaction {
            increment(b);
        }
    });

    let _ = t1.join();
    let _ = t2.join();

    assert_eq!(*a_saved.borrow(), 1);
    assert_eq!(*b_saved.borrow(), 1);
}

/// T1: A
/// T2: B
/// T3: A
/// T4: A, B
#[test]
fn two_in_tx() {
    let a = Arc::new(TxPtr::new(0));
    let a_saved = a.clone();

    let b = Arc::new(TxPtr::new(0));
    let b_clone = b.clone();
    let b_saved = b.clone();

    let t1 = thread::spawn(move || {
        transaction {
            increment(a);
        }
        transaction {
            increment(b);
        }
    });

    let t2 = thread::spawn(move || {
        transaction {
            increment(b_clone);
        }
    });

    let _ = t1.join();
    let _ = t2.join();

    assert_eq!(*a_saved.borrow(), 1);
    assert_eq!(*b_saved.borrow(), 2);
}

#[test]
///all 3 have overlapping conflict sets
///t1 - a, b
///t2 - b, c
///t3 - a, c
fn triple_overlap() {
    let a = Arc::new(TxPtr::new(0));
    let a_clone = a.clone();
    let a_saved = a.clone();

    let b = Arc::new(TxPtr::new(0));
    let b_clone = b.clone();
    let b_saved = b.clone();

    let c = Arc::new(TxPtr::new(0));
    let c_clone = c.clone();
    let c_saved = c.clone();
    let t1 = thread::spawn(move || {
        transaction {
            let a_ref = a.borrow_mut();
            thread::sleep(time::Duration::from_millis(1));
            *a_ref += 1;
            let b_ref = b.borrow_mut();
            thread::sleep(time::Duration::from_millis(1));
            *b_ref += 1;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let b_ref = b_clone.borrow_mut();
            thread::sleep(time::Duration::from_millis(1));
            *b_ref -= 3;

            let c_ref = c.borrow_mut();
            thread::sleep(time::Duration::from_millis(1));
            *c_ref -= 3;
        }
    });
    let t3 = thread::spawn(move || {
        transaction {
            let a_ref = a_clone.borrow_mut();
            thread::sleep(time::Duration::from_millis(1));
            *a_ref += 4;

            let c_ref = c_clone.borrow_mut();
            thread::sleep(time::Duration::from_millis(1));
            *c_ref += 4;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    let _ = t3.join();
    assert_eq!(*a_saved.borrow(), 5);
    assert_eq!(*b_saved.borrow(), -2);
    assert_eq!(*c_saved.borrow(), 1);
}