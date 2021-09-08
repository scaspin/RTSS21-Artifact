//! Tests that look at how the allocation set for each transaction is determined without
//! creating new threads.
use txcell::TxPtr;

// Just create a TxPtr.
#[test]
fn one_txrefcell() {
    let rc = TxPtr::new(3);
    let rc2 = rc;
    assert_eq!(*rc2.borrow(), 3);
}

// Example with one TxPtr and one transaction (no thread::spawn).
#[test]
fn one_transaction_read() {
    let rc = TxPtr::new(5);
    let out;
    transaction {
        let rc_ref = rc.borrow();
        out = rc_ref.clone();
    }
    assert_eq!(out, 5);
}

// Example with one TxPtr and one transaction (no thread::spawn).
#[test]
fn one_transaction_write() {
    let rc = TxPtr::new(7);
    transaction {
        let rc_ref = rc.borrow_mut();
        *rc_ref -= 3;
    }
    assert_eq!(*rc.borrow(), 4);
}

// Two possible allocation sites.
#[test]
fn alloc_set() {
    let rc;
    if rand::random() {
        rc = TxPtr::new(0);
    } else {
        rc = TxPtr::new(1);
    }
    transaction {
        let rc_ref = rc.borrow_mut();
        *rc_ref += 1;
    }
    assert!(*rc.borrow() == 1 || *rc.borrow() == 2);
}


// TODO: Recognize that only 3 is a possible allocation for the transaction.
#[test]
fn use_in_if() {
    let rc;
    if rand::random() {
        rc = TxPtr::new(5);
    } else {
        rc = TxPtr::new(3);
        transaction {
            let rc_ref = rc.borrow_mut();
            *rc_ref -= 1;
        }
    }
    assert!(*rc.borrow() == 5 || *rc.borrow() == 2);
}
