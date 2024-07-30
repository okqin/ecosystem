use std::{thread, time::Duration};
use tokio::{fs, runtime::Builder, time::sleep};

// tokio runtime example for
fn main() {
    let handle = thread::spawn(|| {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        rt.spawn(async {
            println!("Future 1");
            let content = fs::read_to_string("Cargo.toml").await.unwrap();
            println!("Content length: {}", content.len());
        });

        rt.spawn(async {
            let name = "Future 2";
            println!("{}", name);
            let ret = expensive_blocking_task(name);
            println!("result: {}", ret);
        });

        rt.block_on(async {
            sleep(Duration::from_millis(1000)).await;
        });
    });
    handle.join().unwrap();
}

fn expensive_blocking_task(name: &str) -> String {
    thread::sleep(Duration::from_millis(900));
    format!("{} done", name)
}
