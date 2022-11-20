mod thread_pool;

use std::env;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::OsRng;

// use http::{Request, Response};
#[tokio::main]
async fn main(){
    let args: Vec<String> = env::args().collect();
    
    let ip1 = format!("{}{}","http://" ,&args[1]);
    let ip2 = format!("{}{}","http://" ,&args[2]);
    let ip3 = format!("{}{}","http://" ,&args[3]);
    
    let ips_vec = vec![ip1.clone(), ip2.clone(), ip3.clone()];
    let ips_vec_2 = vec![ip1, ip2, ip3];


    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            let mut rng = ::rand::rngs::StdRng::from_seed(OsRng.gen());
            let random_index = rng.gen_range(0..=2);
            println!("Thread spawned");
            
            let response = send_request(ips_vec[1].clone()).await;
            println!("Sent request to server {}", random_index);
            println!("Init election response={:?}", response);
        }
    });

    let mut i = 0;
    let client = reqwest::Client::new();
    loop {
        
        // let _request = client.get(ips_vec_2[i].clone()).header("fn", "ping")
        // .send()
        // .await;

        i = (i+1)%3;

    }
}

async fn send_request(ip:String) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let request = client.get(ip).header("fn", "init_election")
    .header("id", "1")
    .send().await;
    
    // let body = client.get("http://127.0.0.1:7878").send()

    Ok("Called".to_string())
}