extern crate futures;
extern crate tokio_core;

use tokio_core::{Sender, Receiver};
use tokio_core::io::IoFuture;
use futures::stream::Stream;
use futures::{Future,finished};
use std::thread::spawn;
use std::io;

fn spawn_aux(trx: Sender<u32>, rx:  IoFuture<Receiver<u32>>) {
    spawn(move || {
        let mut aux_loop = tokio_core::Loop::new().unwrap();
        let future = rx.and_then(|s| {
            s.for_each(|num| {
                trx.send(num + 1)
            })
        });
        aux_loop.run(future)
    });
}

fn main() {
    let mut main_loop = tokio_core::Loop::new().unwrap();
    let (first_trx, mut last_rx) = main_loop.handle().channel::<u32>();
    for _ in  1..2 {
        let (trx2, rx2) = main_loop.handle().channel::<u32>();
        spawn_aux(trx2, last_rx);
        last_rx = rx2
    }
    first_trx.send(0).unwrap();
    let future = last_rx.and_then(|s| {
        s.take(1_000_000)
         .fold(0, |_, num|{
             let num = num + 1;
             first_trx.send(num).unwrap();
             finished::<u32, io::Error>(num)
         })
    });
    let res = main_loop.run(future).unwrap();
    println!("res {}", res);
}
