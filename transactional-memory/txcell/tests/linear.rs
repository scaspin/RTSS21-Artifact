#![feature(test)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crossbeam_utils::thread;
use rand::distributions::{Bernoulli, Distribution, Uniform};
use std::convert::TryInto;
use std::sync::Arc;
use std::time::{Duration, Instant};
use swym::{tcell::TCell, thread_key, tx::Ordering};
use txcell::TxPtr;

const NUM_RAND: usize = 200;

/// - spawn N threads
/// - for each thread:
///     - precompute random numbers
///     - start timer
///     - repeat the following transaction
///         - for each of N elements:
///         - read or write the element depending on random number
///     - stop timer
/// - sum timers
fn test_stm(
    buf_size: usize,
    num_cores: usize,
    num_accesses: usize,
    percent_writes: f64,
    min_duration: Duration,
) -> (Duration, usize) {
    thread::scope(|scope| {
        let mut buf = Vec::new();
        for _ in 0..buf_size {
            buf.push(TxPtr::new(0))
        }
        let buf = Arc::new(buf);
        let mut longest_time = Duration::new(0, 0);
        let mut total_ops = 0;
        let mut handles = vec![];
        // spawn N threads
        for m in 0..num_cores {
            let buf_clone = Arc::clone(&buf);
            handles.push(scope.spawn(move |_| {
                unsafe {
                    let thread_id = gettid();
                    let cpu: i32 = m.try_into().unwrap();
                    if be_migrate_thread_to_cpu(thread_id, cpu) != 0 {
                        println!("migration error >:(");
                    }
                }
                // precompute random numbers
                let mut rng = rand::thread_rng();
                let write_distribution = Bernoulli::new(percent_writes).unwrap();
                let index_distribution = Uniform::new(0, buf_size);
                let mut write_sets = Vec::with_capacity(NUM_RAND);
                let mut index_sets = Vec::with_capacity(NUM_RAND);
                for _ in 0..NUM_RAND {
                    let mut is_write = Vec::with_capacity(num_accesses);
                    let mut indices = Vec::with_capacity(num_accesses);
                    for _ in 0..num_accesses {
                        let write = write_distribution.sample(&mut rng);
                        is_write.push(write);
                        let index = index_distribution.sample(&mut rng);
                        indices.push(index);
                    }
                    write_sets.push(is_write);
                    index_sets.push(indices);
                }
                let mut iter_duration = Duration::new(0, 0);
                let mut num_iterations = 0;
                // repeat the following transaction
                let mut _count = 0;
                while iter_duration < min_duration {
                    let is_write = &write_sets[num_iterations % NUM_RAND];
                    let indices = &index_sets[num_iterations % NUM_RAND];
                    let all_reads = !is_write.iter().any(|w| *w);
                    // Start timer
                    let now;
                    if all_reads {
                        now = Instant::now();
                        transaction {
                            // For each of the N elements
                            for i in 0..num_accesses {
                                let index = indices[i];
                                let buf_i = &buf_clone[index];
                                _count += *buf_i.borrow() / 10_000;
                            }
                        }
                    } else {
                        now = Instant::now();
                        transaction {
                            // For each of the N elements
                            for i in 0..num_accesses {
                                let index = indices[i];
                                let buf_i = &buf_clone[index];
                                // Read or write element depending on random number
                                if is_write[i] {
                                    *buf_i.borrow_mut() += 1;
                                } else {
                                    _count += *buf_i.borrow() / 10000;
                                }
                            }
                        }
                    }
                    // Stop timer
                    iter_duration += now.elapsed();
                    num_iterations += 1;
                }
                (iter_duration, num_iterations)
            }));
        }
        for handle in handles {
            let (thread_time, thread_ops) = handle.join().unwrap();
            if thread_time > longest_time {
                longest_time = thread_time;
            }
            total_ops += thread_ops;
        }
        (longest_time, total_ops)
    })
    .unwrap()
}

/// - spawn N threads
/// - for each thread:
///     - precompute random numbers
///     - start timer
///     - repeat the following transaction 10K times
///         - for each of N elements:
///         - read or write the element depending on random number
///     - stop timer
/// - sum timers
fn test_swym(
    buf_size: usize,
    num_cores: usize,
    num_accesses: usize,
    percent_writes: f64,
    min_duration: Duration,
) -> (Duration, usize) {
    thread::scope(|scope| {
        let mut buf = Vec::new();
        for _ in 0..buf_size {
            buf.push(TCell::new(0))
        }
        let buf = Arc::new(buf);
        let mut longest_time = Duration::new(0, 0);
        let mut total_ops = 0;
        let mut handles = vec![];
        // spawn N threads
        for m in 0..num_cores {
            let buf_clone = Arc::clone(&buf);
            handles.push(scope.spawn(move |_| {
                unsafe {
                    let thread_id = gettid();
                    let cpu: i32 = m.try_into().unwrap();
                    if be_migrate_thread_to_cpu(thread_id, cpu) != 0 {
                        println!("migration error >:(");
                    }
                }
                // precompute random numbers
                let mut rng = rand::thread_rng();
                let write_distribution = Bernoulli::new(percent_writes).unwrap();
                let index_distribution = Uniform::new(0, buf_size);
                let mut write_sets = Vec::with_capacity(NUM_RAND);
                let mut index_sets = Vec::with_capacity(NUM_RAND);
                for _ in 0..NUM_RAND {
                    let mut is_write = Vec::with_capacity(num_accesses);
                    let mut indices = Vec::with_capacity(num_accesses);
                    for _ in 0..num_accesses {
                        let write = write_distribution.sample(&mut rng);
                        is_write.push(write);
                        let index = index_distribution.sample(&mut rng);
                        indices.push(index);
                    }
                    write_sets.push(is_write);
                    index_sets.push(indices);
                }
                let mut iter_duration = Duration::new(0, 0);
                let mut num_iterations = 0;
                // repeat the following transaction
                let thread_key = thread_key::get();
                while iter_duration < min_duration {
                    let mut count = 0;
                    let is_write = &write_sets[num_iterations % NUM_RAND];
                    let indices = &index_sets[num_iterations % NUM_RAND];
                    let all_reads = !is_write.iter().any(|w| *w);
                    // start timer
                    let now;
                    if all_reads {
                        now = Instant::now();
                        thread_key.read(|tx| {
                            // For each of the N elements
                            for i in indices {
                                let buf_i = &buf_clone[*i];
                                let val = buf_i.get(tx, Ordering::default())?;
                                count += val;
                            }
                            Ok(count)
                        });
                    } else {
                        now = Instant::now();
                        thread_key.rw(|tx| {
                            // For each of the N elements
                            for i in 0..num_accesses {
                                let index = indices[i];
                                let buf_i = &buf_clone[index];
                                // Read or write depending on random number
                                if is_write[i] {
                                    let next = buf_i.get(tx, Ordering::default())? + 1;
                                    buf_i.set(tx, next)?;
                                } else {
                                    let val = buf_i.get(tx, Ordering::default())?;
                                    count += val;
                                }
                            }
                            Ok(count)
                        });
                    }
                    // Stop timer
                    iter_duration += now.elapsed();
                    num_iterations += 1;
                }
                (iter_duration, num_iterations)
            }));
        }
        for handle in handles {
            let (thread_time, thread_ops) = handle.join().unwrap();
            if thread_time > longest_time {
                longest_time = thread_time;
            }
            total_ops += thread_ops;
        }
        (longest_time, total_ops)
    })
    .unwrap()
}

fn ops_per_second(duration: Duration, num_ops: usize) -> f64 {
    // number of operations = number of transactions.
    let duration_sec = duration.as_secs_f64();
    num_ops as f64 / duration_sec
}

#[test]
fn linear1() {
    let now = Instant::now();
    // N size of buffer
    let buf_sizes = vec![64, 128, 256];
    // M cores, 1 thread per core
    let num_cores = vec![
        1, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36,
    ];
    // X% of buffer elements are accessed
    let percent_accessed = vec![0.1];
    // Y% of accesses are writes
    let percent_writes = 0.05;
    // Repeat this many times
    let min_duration = Duration::from_secs(3);
    // output in CSV format
    println!("buf_size,num_cores,percent_accessed,percent_writes,stm_ops_per_sec,swym_ops_per_sec,stm_ops,swym_ops");
    for n in buf_sizes {
        for m in &num_cores {
            for x_pct in &percent_accessed {
                let num_accesses = (x_pct * n as f64) as usize;
                let (duration_stm, ops_stm) =
                    test_stm(n, *m, num_accesses, percent_writes, min_duration);
                let tput_stm = ops_per_second(duration_stm, ops_stm);

                let (duration_swym, ops_swym) =
                    test_swym(n, *m, num_accesses, percent_writes, min_duration);
                let tput_swym = ops_per_second(duration_swym, ops_swym);

                println!(
                    "{},{},{},{},{},{}",
                    n, m, x_pct, percent_writes, tput_stm, tput_swym
                );
            }
        }
    }
    let elapsed = now.elapsed();
    println!(
        "Test complete. Took {:?} ... test linear1 has been running for over 60 seconds",
        elapsed
    );
}

#[test]
fn linear2() {
    let now = Instant::now();
    // N size of buffer
    let buf_sizes = vec![64, 128, 256];
    // M cores, 1 thread per core
    let num_cores = vec![8, 16];
    // X% of buffer elements are accessed
    let percent_accessed = [
        0.05, 0.10, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8,
        0.85, 0.9, 0.95, 1.0,
    ];
    // Y% of accesses are writes
    let percent_writes = 0.05;
    // Repeat this many times
    let min_duration = Duration::from_secs(3);
    // output in CSV format
    println!("buf_size,num_cores,percent_accessed,percent_writes,stm_ops_per_sec,swym_ops_per_sec,stm_ops,swym_ops");
    for n in buf_sizes {
        for m in &num_cores {
            for x in &percent_accessed {
                let num_accesses = (x * n as f64) as usize;
                let (duration_stm, ops_stm) =
                    test_stm(n, *m, num_accesses, percent_writes, min_duration);
                let tput_stm = ops_per_second(duration_stm, ops_stm);

                let (duration_swym, ops_swym) =
                    test_swym(n, *m, num_accesses, percent_writes, min_duration);
                let tput_swym = ops_per_second(duration_swym, ops_swym);

                println!(
                    "{},{},{},{},{},{},{},{}",
                    n, *m, x, percent_writes, tput_stm, tput_swym, ops_stm, ops_swym
                );
            }
        }
    }
    let elapsed = now.elapsed();
    println!(
        "Test complete. Took {:?} ... test linear2 has been running for over 60 seconds",
        elapsed
    );
}

#[test]
fn linear3() {
    let now = Instant::now();
    // N size of buffer
    let buf_sizes = vec![64, 128, 256];
    // M cores, 1 thread per core
    let num_cores = vec![8, 16];
    // X% of buffer elements are accessed
    let percent_accessed = 0.05;
    // Y% of accesses are writes
    let percent_writes = [
        0.0, 0.05, 0.10, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55, 0.6, 0.65, 0.7, 0.75,
        0.8, 0.85, 0.9, 0.95, 1.0,
    ];
    // Repeat this many times
    let min_duration = Duration::from_secs(3);
    // output in CSV format
    println!("buf_size,num_cores,num_accesses,percent_writes,stm_ops_per_sec,swym_ops_per_sec");
    for n in buf_sizes {
        for m in &num_cores {
            for y in &percent_writes {
                let num_accesses = (percent_accessed * n as f64) as usize;
                let (duration_stm, ops_stm) = test_stm(n, *m, num_accesses, *y, min_duration);
                let tput_stm = ops_per_second(duration_stm, ops_stm);

                let (duration_swym, ops_swym) = test_swym(n, *m, num_accesses, *y, min_duration);
                let tput_swym = ops_per_second(duration_swym, ops_swym);

                println!(
                    "{},{},{},{},{},{}",
                    n, *m, num_accesses, y, tput_stm, tput_swym
                );
            }
        }
    }
    let elapsed = now.elapsed();
    println!(
        "Test complete. Took {:?} ... test linear3 has been running for over 60 seconds",
        elapsed
    );
}
