//!Tests that compare how reading and writing affect locking.
use std::sync::Arc;
use std::{thread, time};
use txcell::TxPtr;

#[test]
///all 3 have overlapping conflict sets only if reads are counted
///will always pass
fn read_write_overlap() {
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
            *a_ref += 5;
            let b_ref = b.borrow_mut();
            thread::sleep(time::Duration::from_millis(1));
            *b_ref += 5;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let b_ref = b_clone.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let c_ref = c.borrow_mut();
            thread::sleep(time::Duration::from_millis(1));
            *c_ref += 5;
            let _ = *b_ref;
        }
    });
    let t3 = thread::spawn(move || {
        transaction {
            let a_ref = a_clone.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let c_ref = c_clone.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let _ = *a_ref + *c_ref;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    let _ = t3.join();
    assert_eq!(*a_saved.borrow(), 5);
    assert_eq!(*b_saved.borrow(), 5);
    assert_eq!(*c_saved.borrow(), 5);
}

#[test]
///all threads only read, will always pass
fn read_only_overlap() {
    let a = Arc::new(TxPtr::new(0));
    let a_clone = a.clone();
    let a_clone2 = a.clone();
    let a_saved = a.clone();

    let b = Arc::new(TxPtr::new(0));
    let b_clone = b.clone();
    let b_clone2 = b.clone();
    let b_saved = b.clone();

    let c = Arc::new(TxPtr::new(0));
    let c_clone = c.clone();
    let c_clone2 = c.clone();
    let c_saved = c.clone();
    let t1 = thread::spawn(move || {
        transaction {
            let ta = a.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let tb = b.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let tc = c.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let _ = *ta + *tb + *tc;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let ta = a_clone.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let tb = b_clone.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let tc = c_clone.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let _ = *ta + *tb + *tc;
        }
    });
    let t3 = thread::spawn(move || {
        transaction {
            let ta = a_clone2.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let tb = b_clone2.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let tc = c_clone2.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let _ = *ta + *tb + *tc;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    let _ = t3.join();
    assert_eq!(*a_saved.borrow(), 0);
    assert_eq!(*b_saved.borrow(), 0);
    assert_eq!(*c_saved.borrow(), 0);
}

#[test]
///only T1 writes
fn one_writer_overlap() {
    let a = Arc::new(TxPtr::new(0));
    let a_clone = a.clone();
    let a_clone2 = a.clone();
    let a_saved = a.clone();

    let b = Arc::new(TxPtr::new(0));
    let b_clone = b.clone();
    let b_clone2 = b.clone();
    let b_saved = b.clone();

    let c = Arc::new(TxPtr::new(0));
    let c_clone = c.clone();
    let c_clone2 = c.clone();
    let c_saved = c.clone();
    let t1 = thread::spawn(move || {
        transaction {
            let a_ref = a.borrow_mut();
            *a_ref = 3;
            thread::sleep(time::Duration::from_millis(1));
            let b_ref = b.borrow_mut();
            *b_ref = 4;
            thread::sleep(time::Duration::from_millis(1));
            let c_ref = c.borrow_mut();
            *c_ref = 5;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let a_ref = a_clone.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let b_ref = b_clone.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let c_ref = c_clone.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let _ = *a_ref + *b_ref + *c_ref;
        }
    });
    let t3 = thread::spawn(move || {
        transaction {
            let a_ref = a_clone2.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let b_ref = b_clone2.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let c_ref = c_clone2.borrow();
            thread::sleep(time::Duration::from_millis(1));
            let _ = *a_ref + *b_ref + *c_ref;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    let _ = t3.join();
    assert_eq!(*a_saved.borrow(), 3);
    assert_eq!(*b_saved.borrow(), 4);
    assert_eq!(*c_saved.borrow(), 5);
}

#[test]
///all threads write but no reads
/// T1: A, B
/// T2: B, C
/// T3: A, C
fn write_only_overlap() {
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
            *a_ref = 3;
            thread::sleep(time::Duration::from_millis(1));
            let b_ref = b.borrow_mut();
            *b_ref = 4;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let b_ref = b_clone.borrow_mut();
            *b_ref = 3;
            thread::sleep(time::Duration::from_millis(1));
            let c_ref = c.borrow_mut();
            *c_ref = 4;
        }
    });
    let t3 = thread::spawn(move || {
        transaction {
            let c_ref = c_clone.borrow_mut();
            *c_ref = 3;
            thread::sleep(time::Duration::from_millis(1));
            let a_ref = a_clone.borrow_mut();
            *a_ref = 4;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    let _ = t3.join();

    let a_result = *a_saved.borrow();
    assert!(a_result == 3 || a_result == 4);
    let b_result = *b_saved.borrow();
    assert!(b_result == 3 || b_result == 4);
    let c_result = *c_saved.borrow();
    assert!(c_result == 3 || c_result == 4);
}
