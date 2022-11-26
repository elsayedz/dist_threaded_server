use std::env;
use std::io::Read;
use std::net::TcpListener;
use std::thread::sleep;
use std::time::Duration;

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
    let init_election_flag = &args[4].parse::<usize>().unwrap();
    

    let ips_vec = vec![ip1.clone(), ip2.clone(), ip3.clone()];
    let ips_vec_2 = vec![ip1, ip2, ip3];

    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    let client = reqwest::Client::new();

    
    let var:&usize =&1;
    if init_election_flag == var {
        tokio::task::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                let mut rng = ::rand::rngs::StdRng::from_seed(OsRng.gen());
                let random_index = rng.gen_range(0..=2);
                println!("Thread spawned");
                println!("Trying to send to server {}", ips_vec[random_index]);
                
                let response = send_request(ips_vec[random_index].clone()).await;
                println!("Sent request to server {}", random_index);
                println!("Init election response={:?}", response);
            }
        });
    }

    let mut i = 0;

    
    // set timeout for client
    loop {
        
        for stream in listener.incoming(){
            let mut stream = stream.unwrap();
            println!("Request received");
            let mut buffer = [0; 1024];
            stream.read(&mut buffer).unwrap();
            
            let _request = client.get(ips_vec_2[i].clone())
            .header("fn", "ping")
            .timeout(Duration::from_secs(5))
            .send()
            .await;
            sleep(Duration::from_secs(1));
            i = (i+1)%3;
        }

    }
}

async fn send_request(ip:String) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let _request = client.get(ip).header("fn", "init_election")
    .header("id", "1")
    .send().await;
    
    Ok("Called".to_string())
}