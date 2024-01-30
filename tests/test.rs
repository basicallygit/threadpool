use threadpool::ThreadPool;

use std::time::Duration;
use std::sync::mpsc;
use std::thread;

#[test]
fn test_threadpool_execute() {
    let pool = ThreadPool::new(3).unwrap();

    let (sender, reciever) = mpsc::channel();
    
    let mut sleep_millis = 250;
    for i in 1..=4 {
        let sender = sender.clone();
        pool.execute(move || {
            thread::sleep(Duration::from_millis(sleep_millis));
            sender.send(i).unwrap();
        });

        sleep_millis += 250;
    }

    let mut numbers = vec![];
    for _ in 1..=4 {
        let num = reciever.recv().unwrap();
        numbers.push(num);
    }

    assert_eq!(numbers, vec![1, 2, 3, 4]);
}

#[test]
#[should_panic]
fn test_zero_size_threadpool() {
    let _pool = ThreadPool::new(0);
}
