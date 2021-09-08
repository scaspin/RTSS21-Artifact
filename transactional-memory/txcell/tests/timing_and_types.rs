//!Tests that look at how time delays and different types affect transactions.
use std::sync::Arc;
use std::{thread, time};
use txcell::TxPtr;

// Two threads use the same TxPtr.
#[test]
fn id_simple() {
    let a = Arc::new(TxPtr::new(0));
    let a_clone = a.clone();
    let a_saved = a.clone();
    let t1 = thread::spawn(move || {
        transaction {
            let t = a.borrow_mut();
            *t += 1;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let t = a_clone.borrow_mut();
            *t -= 1;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    assert_eq!(*a_saved.borrow(), 0);
}

// An increment and decrement with a 16-bit number
#[test]
fn id_med_number() {
    //16 bit number
    let a = Arc::new(TxPtr::new(65000));
    let a_clone = a.clone();
    let a_saved = a.clone();
    let t1 = thread::spawn(move || {
        transaction {
            let t = a.borrow_mut();
            *t += 5;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let t = a_clone.borrow_mut();
            *t -= 3;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    assert!(*a_saved.borrow() == 65002);
}

// An increment and decrement with a 32-bit number
#[test]
fn id_big_number() {
    //32 bit number
    let a = Arc::new(TxPtr::new(420_000_000));
    let a_clone = a.clone();
    let a_saved = a.clone();
    let t1 = thread::spawn(move || {
        transaction {
            let t = a.borrow_mut();
            *t += 5;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let t = a_clone.borrow_mut();
            *t -= 3;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    assert!(*a_saved.borrow() == 420_000_002);
}

// An increment and decrement with an added 1ms delay between reads and writes
// Simulates what might happen when performing more complex operations
#[test]
fn id_small_wait() {
    let a = Arc::new(TxPtr::new(0));
    let a_clone = a.clone();
    let a_saved = a.clone();
    let t1 = thread::spawn(move || {
        transaction {
            let t = a.borrow_mut();
            thread::sleep(time::Duration::from_millis(1));
            *t += 5;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let t = a_clone.borrow_mut();
            thread::sleep(time::Duration::from_millis(1));
            *t -= 3;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    assert!(*a_saved.borrow() == 2);
}

// An increment and decrement with an added 10ms delay between reads and writes
// Simulates what might happen when performing more complex operations
// Should fail more often than the 1ms delay
#[test]
fn id_big_wait() {
    let a = Arc::new(TxPtr::new(0));
    let a_clone = a.clone();
    let a_saved = a.clone();
    let t1 = thread::spawn(move || {
        transaction {
            let t = a.borrow_mut();
            thread::sleep(time::Duration::from_millis(10));
            *t += 5;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let t = a_clone.borrow_mut();
            thread::sleep(time::Duration::from_millis(10));
            *t -= 3;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    assert_eq!(*a_saved.borrow(), 2);
}

#[derive(Clone, Copy)]
struct Foo {
    x: bool,
    y: i32,
}

#[test]
fn struct_simple() {
    let a = Arc::new(TxPtr::new(Foo { x: true, y: 3 }));
    let a_clone = a.clone();
    let a_saved = a.clone();
    let t1 = thread::spawn(move || {
        transaction {
            let t = a.borrow_mut();
            thread::sleep(time::Duration::from_millis(10));
            t.y += 3;
        }
    });
    let t2 = thread::spawn(move || {
        transaction {
            let t = a_clone.borrow_mut();
            thread::sleep(time::Duration::from_millis(10));
            t.y -= 3;
        }
    });
    let _ = t1.join();
    let _ = t2.join();
    let out = *a_saved.borrow();
    assert_eq!(out.x, true);
    assert_eq!(out.y, 3);
}
