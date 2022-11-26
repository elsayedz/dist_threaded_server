use std::thread::sleep;
use std::time::Duration;


#[tokio::main]
async fn main(){
        
    let client = reqwest::Client::new();
    loop {
        
        let _request = client.get("http://127.0.0.1:8000")
        .header("fn", "ping")
        .timeout(Duration::from_secs(5))
        .send()
        .await;
        println!("Sent request");
        sleep(Duration::from_secs(1));
    }
}