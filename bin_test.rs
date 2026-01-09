use std::net::TcpListener;

fn main() {
    match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => {
            println!("bound to {:?}", listener.local_addr());
        }
        Err(err) => {
            eprintln!("bind failed: {err}");
        }
    }
}
