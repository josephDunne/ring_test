extern crate futures;
extern crate tokio_core;

use tokio_core::reactor::Core;
use tokio_core::channel::{Sender, channel};
use futures::stream::Stream;
use futures::finished;
use std::thread::spawn;
use std::io;
use std::sync::{Arc,Mutex, Barrier};

fn spawn_aux(trx: Sender<u32>, result: Arc<Mutex<Option<Sender<u32>>>>, barrier: Arc<Barrier>) {
    spawn(move || {
        let mut aux_loop = Core::new().unwrap();
        let (trx2, rx) = channel::<u32>(&aux_loop.handle()).unwrap();
        let future = rx.for_each(|s| {
                trx.send(s + 1)
        });
        {
            let mut data = result.lock().unwrap();
            *data = Some(trx2);
        }
        barrier.wait();
        aux_loop.run(future)
    });
}

fn main() {
    let mut main_loop = Core::new().unwrap();
    let (mut last_trx, last_rx) = channel::<u32>(&main_loop.handle()).unwrap();
    last_trx.send(1).unwrap();
    let sync_mutex = Arc::new(Mutex::new(None));
    for _ in  1..8 {
        let barrier = Arc::new(Barrier::new(2));
        spawn_aux(last_trx, sync_mutex.clone(), barrier.clone());
        barrier.wait();
        last_trx = sync_mutex.lock().unwrap().take().unwrap();
    }
    let future = last_rx.take(1_000_000).fold(0, |_, num| {
        let num = num + 1;
        last_trx.send(num).unwrap();
        finished::<u32, io::Error>(num)
    });
    let res = main_loop.run(future).unwrap();
    println!("res {}", res);
}
