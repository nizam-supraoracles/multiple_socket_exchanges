use std::error::Error;

use ws_socket::start;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let r = start().await;
    if r.is_err() {
        println!("Error: {:?}", r.unwrap_err());
    }
    Ok(())
}
