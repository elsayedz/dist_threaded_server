mod thread_pool;
use thread_pool::ThreadPool;

use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::OsRng;

use std::time::Duration;
use std::fs;
use std::thread;

use httparse;

// use http::{Request, Response};
#[tokio::main]
async fn main(){
    

    //send an init election request each 6 seconds
    let s = send_request().await;
    println!("fact = {:#?}", s);

    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(6));
        loop {
            interval.tick().await;
            let mut rng = ::rand::rngs::StdRng::from_seed(OsRng.gen());
            let random_index = rng.gen_range(0..=3);
            println!("Thread spawned");
            
            let response = send_request().await;
            println!("Sent request to server {}", random_index);
            println!("Init election response={:?}", response);
        }
    });

    //this is the ping servers loop
    loop {
        
        // let s = send_request().await;
        // println!("fact = {:#?}", s);
        
    }
}

async fn send_request() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let request = client.get("http://10.40.39.94:50050").header("FN_Name", "sayed")
    .header("FN_Name", "init_election")
    .send()
    .await?
    .text()
    .await?;
    // let body = client.get("http://127.0.0.1:7878").send()

    Ok(request)
}