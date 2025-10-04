use std::time::Duration;

async fn count_to(num: u32, counter_name: &str) {
    for i in 0..num {
        println!("{} : {}", &counter_name, i);
        tokio::time::sleep(Duration::from_secs(1)).await
    }
    println!("{} finished", counter_name)
}

#[tokio::main]
async fn main() {
    println!("starting program");

    tokio::select! {

        a = count_to(3, "first") => {

        }
        b = count_to(10, "second") => {

        }
    }

    println!("I am done on main thread");

    tokio::time::sleep(Duration::from_secs(11)).await
}
