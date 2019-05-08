fn main() {
    match mone::run() {
        Ok(()) => (),
        Err(err) => println!("{:?}", err),
    }
}
