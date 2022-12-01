use std::{env, thread};
use std::thread::sleep;
use std::time::Duration;

use std::fs::OpenOptions;
use std::io::{Write, Seek};

use tokio::task;


#[tokio::main]
async fn main(){
    let args: Vec<String> = env::args().collect();
    let client_num = &args[1];
    
        println!("Task spawned");
        let client = reqwest::Client::new();
        let mut i :usize = 1;
        
        let path = format!("results/client{}secondcopy.txt", client_num);
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)
            .unwrap();

        let mut _average_response_time = 0.0;
        let mut total_response_time = 0.0;
        let mut batch_number = 1.0;
        let mut number_of_failed_requests = 0;
        for _ in 0..1000 {
            let mut response_time = 0.0;
            let start = std::time::Instant::now();
            let _request = client.get("http://127.0.0.1:8000")
            .header("fn", "ping")
            .header("id", i.to_string())
            .send()
            .await;
            
            println!("Sent request");
            response_time += start.elapsed().as_millis() as f64;
            total_response_time += response_time;
            let mut response = String::new();
            match _request {
                Ok(_) => {
                    response = "OK".to_string();
                },
                Err(e) => {
                    
                    number_of_failed_requests += 1;
                    if e.is_timeout() {
                        println!("Timeout");
                        response = "Timeout".to_string();
                    }
                    else {
                        println!("Error: {}", e);
                        
                    }
                }
            }
            if let Err(e) = writeln!(file, "Request {}: Response: {:?}, response time: {} ms Total failed requests: {:?}", i, response, response_time, number_of_failed_requests) {
                eprintln!("Couldn't write to file: {}", e);
            }
            if i%100 == 0 {
                _average_response_time = total_response_time /( 100.0 * batch_number);
                //clear file contents
                file.set_len(0).unwrap();
                file.rewind().unwrap();
                if let Err(e) = writeln!(file, "Average response time: {} ms for batch number {} Total Failed Requests: {}", _average_response_time, batch_number, number_of_failed_requests) {
                    eprintln!("Couldn't write to file: {}", e);
                }
                batch_number += 1.0;
            }
            sleep(Duration::from_millis(100));
            i+=1;
        }
      
    
}