use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use txcell::TxPtr;

const COUNT: usize = 1000;

/// Two queues. Producer produces to `qout`, mover moves from `qout` to `qin`,
/// consumer consumes from `qin`.
#[test]
fn two_queues() {
    let qin: Arc<TxPtr<VecDeque<usize>>> = Arc::new(TxPtr::new(VecDeque::with_capacity(COUNT)));
    let qout: Arc<TxPtr<VecDeque<usize>>> = Arc::new(TxPtr::new(VecDeque::with_capacity(COUNT)));

    // The producer task
    let qout_clone = Arc::clone(&qout);
    let producer_task = thread::spawn(move || {
        let mut cnt = 0;
        while cnt < COUNT {
            transaction {
                // if queue is not full
                if qout_clone.borrow().len() < COUNT {
                    cnt += 1;
                    // enqueue
                    qout_clone.borrow_mut().push_back(cnt);
                }
            }
        }
    });

    // The consumer task
    let qin_clone = Arc::clone(&qin);
    let consumer_task = thread::spawn(move || {
        let mut cnt = 0;
        while cnt < COUNT {
            transaction {
                if let Some(_obj) = qin_clone.borrow_mut().pop_front() {
                    cnt += 1;
                }
            }
        }
    });

    // The mover task
    let mover_task = thread::spawn(move || {
        let mut cnt = 0;
        while cnt < COUNT {
            transaction {
                if qin.borrow().len() < COUNT {
                    // if in is not full
                    if let Some(obj) = qout.borrow_mut().pop_front() {
                        // enqueue
                        qin.borrow_mut().push_back(obj);
                        cnt += 1;
                    }
                }
            }
        }
    });

    let _ = producer_task.join();
    let _ = consumer_task.join();
    let _ = mover_task.join();
}
