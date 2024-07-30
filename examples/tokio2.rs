// Passing messages between asynchronous and synchronous threads.
use anyhow::Result;
use std::{thread, time::Duration};
use tokio::sync::mpsc;

fn worker(mut rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(msg) = rx.blocking_recv() {
            let ret = expensive_blocking_task(&msg);
            println!("result: {}", ret);
        }
    })
}

fn expensive_blocking_task(name: &str) -> String {
    thread::sleep(Duration::from_millis(900));
    format!("{} done", name)
}

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(32);
    let handle = worker(rx);

    tokio::spawn(async move {
        let mut number = 0;
        loop {
            number += 1;
            println!("Sending message {number}");
            tx.send(format!("message {number}")).await?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });
    handle.join().unwrap();
    Ok(())
}
