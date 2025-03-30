use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let should_throw = args.contains(&"--throw".to_owned());
    let should_run_forever = args.iter().any(|s| s == "--forever");

    match (should_throw, should_run_forever) {
        (false, true) => run_forever(),
        (true, _) => {
            println!("I'm running but will throw!");
            run_and_stop();
            panic!("bing bong!");
        },
        _ => {
            println!("I'm running!");
            run_and_stop()
        },
    }
}

fn run_and_stop() {
    for i in 1..5 {
        println!("{}: Hello, world @ {:?}", i, datetime::Instant::now());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn run_forever() {
    println!("I'm running forever!");
    let mut i = 0;
    loop {
        println!("{}: Hello, world @ {:?}", i, datetime::Instant::now());
        std::thread::sleep(std::time::Duration::from_secs(1));
        i += 1;
    }
}