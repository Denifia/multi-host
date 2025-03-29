fn main() {
    loop {
        println!("Hello, world @ {:?}", datetime::Instant::now());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
