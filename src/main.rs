extern crate futures;
extern crate tokio_core;

use std::{
    thread,
    time,
};

use tokio_core::reactor::{
    Core,
    Timeout,
};

use futures::sync::mpsc;
use futures::{
    // future,
    Future,
    Stream,
};

#[derive(Debug)]
struct Item(i32);

impl Drop for Item {
    fn drop(&mut self) {
        println!("  |-> dropped {:?}", self);
    }
}

#[derive(Debug)]
enum Message {
    Proc(i32),
    Halt,
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let (tx, rx) = mpsc::unbounded();

    let t = thread::spawn(move || {
        for i in 0..3 {
            let m = Message::Proc(i);
            println!("prod: sending message {}", i);
            if let Err(r) = tx.unbounded_send(m) {
                println!("error while sending message {:?}", r);
            }
            thread::sleep(time::Duration::from_secs(1));
        }

        println!("prod: sending `Halt` message");
        if let Err(r) = tx.unbounded_send(Message::Halt) {
            println!("fialed to send final message {:?}", r);
        }

        // panic!("shit");
    });

    let to_run = rx.for_each(|m| match m {
        Message::Proc(i) => {
            println!("cons: posting message for item {}", i);

            let item = Item(i);
            let to_proc = Timeout::new(time::Duration::from_secs(5), &handle)
                .unwrap()
                .then(move |_| {
                    let item = item;
                    println!("  |-> processing {:?}", item);
                    Ok(())
                });

            handle.spawn(to_proc);

            Ok(())
        },
        Message::Halt => {
            println!("got `Halt` message");
            Err(())
        }
    });

    match core.run(to_run) {
        Ok(_) => panic!("broken logic within main loop"),
        Err(_) => println!("loop done")
    }

    t.join().unwrap();
}
