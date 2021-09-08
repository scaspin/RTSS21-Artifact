#![feature(lang_items)]
use pflock::PFLock;
use std::cell::{Cell, UnsafeCell};
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod tree;

/// TODO: Docs here
///
/// [`new`]: #method.new
/// [`set`]: #method.set
/// [`get`]: #method.get
///

struct TicketLock {
    now_serving: AtomicUsize,
    next_ticket: AtomicUsize,
}

impl TicketLock {
    const fn new() -> Self {
        TicketLock {
            now_serving: AtomicUsize::new(0),
            next_ticket: AtomicUsize::new(0),
        }
    }

    fn lock(&self) {
        let my_ticket = self.next_ticket.fetch_add(1, Ordering::Relaxed);
        while self.now_serving.load(Ordering::Relaxed) != my_ticket {}
    }

    fn unlock(&self) {
        self.now_serving.store(
            self.now_serving.load(Ordering::Relaxed) + 1,
            Ordering::Relaxed,
        );
    }
}

static mut MUTEXES: [TicketLock; 20] = [
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
    TicketLock::new(),
];

static mut PFLOCKS: [PFLock; 20] = [
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
    PFLock::new(),
];

pub struct TxCell<T>(Cell<T>);

unsafe impl<T> Send for TxCell<T> {}
unsafe impl<T> Sync for TxCell<T> {}

impl<T: Copy> TxCell<T> {
    pub fn new(inner: T) -> TxCell<T> {
        TxCell(Cell::new(inner))
    }

    pub fn set(&self, val: T) {
        self.0.set(val)
    }

    pub fn get(&self) -> T {
        self.0.get()
    }
}

#[derive(Debug)]
pub struct TxPtr<T>(UnsafeCell<T>);

unsafe impl<T> Send for TxPtr<T> {}
unsafe impl<T> Sync for TxPtr<T> {}

impl<T> TxPtr<T> {
    /// Create a new `TxPtr` containing `inner`.
    pub fn new(inner: T) -> TxPtr<T> {
        TxPtr(UnsafeCell::new(inner))
    }

    /// Immutably borrows the wrapped value.
    pub fn borrow(&self) -> &T {
        unsafe { &*self.0.get() }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` or all `RefMuts` derived from it
    /// exit scope. The value cannot be borrowed while this borrow is active.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn borrow_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

/// Simple spinlock. Spin until we set the `AtomicBool` from `false` to `true`.
#[lang = "transaction_lock"]
pub fn lock_mutex(n: usize) {
    unsafe { MUTEXES[n].lock() }
}

/// Simple spinlock. Set the `AtomicBool` to `false`.
#[lang = "transaction_unlock"]
pub fn unlock_mutex(n: usize) {
    unsafe { MUTEXES[n].unlock() }
}

#[lang = "transaction_write_lock"]
pub fn write_lock_mutex(n: usize) {
    unsafe { PFLOCKS[n].write_lock() }
}

#[lang = "transaction_write_unlock"]
pub fn write_unlock_mutex(n: usize) {
    unsafe { PFLOCKS[n].write_unlock() }
}

#[lang = "transaction_read_lock"]
pub fn read_lock_mutex(n: usize) {
    unsafe { PFLOCKS[n].read_lock() }
}

#[lang = "transaction_read_unlock"]
pub fn read_unlock_mutex(n: usize) {
    unsafe { PFLOCKS[n].read_unlock() }
}
