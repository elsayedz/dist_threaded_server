use std::env;
use std::thread::sleep;
use std::time::Duration;

use std::fs::OpenOptions;
use std::io::{Write, Seek};

#[tokio::main]
async fn main(){
        
    let client = reqwest::Client::new();
    let mut i :usize = 1;
    
    let args: Vec<String> = env::args().collect();
    let client_num = &args[1];

    let path = format!("results/client{}.txt", client_num);
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();

    let mut average_response_time = 0;
    let mut batch_number = 1;
    loop {
        let mut response_time = 0;
        let start = std::time::Instant::now();
        let _request = client.get("http://127.0.0.1:8000")
        .header("fn", "ping")
        .header("id", i.to_string())
        .timeout(Duration::from_secs(5))
        .send()
        .await;
        println!("Sent request");
        response_time += start.elapsed().as_millis();
        average_response_time += response_time;
        let mut response = String::new();
        match _request {
            Ok(_) => {
                response = "OK".to_string();
            },
            Err(e) => {
                if e.is_timeout() {
                    println!("Timeout");
                    response = "Timeout".to_string();
                }
                else {
                    println!("Error: {}", e);
                }
            }
        }
        if let Err(e) = writeln!(file, "Request {}: Response: {:?}, response time: {} ms", i, response, response_time) {
            eprintln!("Couldn't write to file: {}", e);
        }
        if i%5000 == 0 {
            average_response_time = average_response_time /( 5000 * batch_number);
            //clear file contents
            file.set_len(0).unwrap();
            file.rewind().unwrap();
            if let Err(e) = writeln!(file, "Average response time: {} ms for batch {}", average_response_time, batch_number) {
                eprintln!("Couldn't write to file: {}", e);
            }
            batch_number += 1;
        }
        sleep(Duration::from_millis(300));
        i+=1;
    }
}