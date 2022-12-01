use std::env;
use std::io::Read;
use std::io::Write;
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
        // Spawn a thread to send an init election request every 1 min
        tokio::task::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut rng = ::rand::rngs::StdRng::from_seed(OsRng.gen());
                let random_index = rng.gen_range(0..=2);
                
                // Every time we choose a random node to init the election
                println!("Sending Election request to server {}", ips_vec[random_index]);
                
                let _response = send_request(ips_vec[random_index].clone()).await;
            }
        });
    }

    let mut i = 0;

    
    
    loop {
        
        // Listen to requests from the client
        for stream in listener.incoming(){
            let mut stream = stream.unwrap();
            let mut buffer = [0; 1024];
            stream.read(&mut buffer).unwrap();
            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut req = httparse::Request::new(&mut headers);
            let parsed_req = req.parse(&buffer);
            let _res;
            // Handle res and check for errors
            match parsed_req {
                Ok(_) => {
                    _res = parsed_req.unwrap();
                },
                Err(_) => {
                    println!("Error parsing request");
                    continue;
                },
            }

            let id = req.headers.iter().find(|h| h.name == "id").unwrap().value;
            println!("id={}", std::str::from_utf8(id).unwrap().to_string());

            let response = client.get(ips_vec_2[i].clone())
            .header("fn", "ping")
            .header("id", id)
            .timeout(Duration::from_secs(1))
            .send()
            .await;

            match response {
                Ok(_response) => {
                    i = (i+1)%3;
                    stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
                    stream.flush().unwrap();
                }
                Err(_e) => {
                    println!("Server {} is down", i);
                    i = (i+1)%3;
                    // If server is down we retry the request to the next server
                    loop {
                        let response = client.get(ips_vec_2[i].clone())
                        .header("fn", "ping")
                        .header("id", id)
                        .timeout(Duration::from_secs(1))
                        .send()
                        .await;
                        match response {
                            Ok(_response) => {
                                i = (i+1)%3;
                                stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
                                break;
                            }
                            Err(_e) => {
                                println!("WRONG");
                                stream.flush().unwrap();
                                i = (i+1)%3;
                            }
                        }
                    }
                }
            }
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