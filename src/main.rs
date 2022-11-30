use httparse::Header;

use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::TcpListener;

use std::sync::Arc;

use std::thread::sleep;
use std::time::Duration;

use httparse;

use tokio::task;
// use tokio::sync::Mutex;
use futures::lock::Mutex;

#[derive(Clone)]
struct ServersInfo {
    id_to_ip: Arc<Mutex<HashMap<String, String>>>,
    pub my_id : Arc<Mutex<String>>,
    pub is_asleep: Arc<Mutex<bool>>,
}

impl ServersInfo {
    fn new(_ip1:String, _ip2:String, _ip3:String, _my_id:String) -> Self {

        let mut ip_id: HashMap<String, String> = HashMap::new();
        
        ip_id.insert("0".to_string(), _ip1.clone());
        ip_id.insert("1".to_string(), _ip2.clone());
        ip_id.insert("2".to_string(), _ip3.clone());
    
        Self {
            id_to_ip: Arc::new(Mutex::new(ip_id)),
            my_id: Arc::new(Mutex::new(_my_id)),
            is_asleep: Arc::new(Mutex::new(false)),
        }
    }
}


#[derive(Clone)]
struct Server {
    servers_info: ServersInfo
}

impl Server {
    fn new(_ip1:String, _ip2:String, _ip3:String, _my_id:String) -> Self {
        Self {
            servers_info: ServersInfo::new(_ip1, _ip2, _ip3, _my_id)
        }
    }

    async fn init_election(&self, headers: & mut [Header<'_>],requester_info:SocketAddr) -> Result<String, Box<dyn std::error::Error>> {
        println!("-------------Init election called from {}:{}-------------", requester_info.ip(), requester_info.port());
        
        let my_id = self.servers_info.my_id.lock().await;
        let mut min_id = std::i32::MAX;
        let map = self.servers_info.id_to_ip.lock().await;

        for (key, _value) in &*map {
            if key.parse::<i32>().unwrap() < min_id {
                min_id = key.parse::<i32>().unwrap();
            }
        }
        println!("min_id: {}", min_id);
        let client = reqwest::Client::new();
        match map.get(&min_id.to_string()){
            Some(ip) => {

                if min_id == my_id.parse::<i32>().unwrap() {
                    drop(my_id);
                    drop(map);
                    self.force_failure(headers, requester_info).await;
                    return Ok("I am the leader".to_string());
                }
                println!("Sending request to: {}", ip);
                let _request = client.get(ip)
                .header("fn", "force_failure")
                .send().await;
               
            },
            None => {
                println!("Couldn't find min id in Map");
            }
        }

        println!("-------------Finished--------------");
        
        Ok(format!("min_id: {}", min_id))

    }

    async fn force_failure(&self, _headers: & mut [Header<'_>],requester_info:SocketAddr) -> Result<String, Box<dyn std::error::Error>> {
        println!("-------------Force failure called from {}:{}-------------", requester_info.ip(), requester_info.port());
        let mut is_asleep = self.servers_info.is_asleep.lock().await;
        *is_asleep = true;
        drop(is_asleep);
        sleep(Duration::from_secs(10));
        let mut is_asleep = self.servers_info.is_asleep.lock().await;
        *is_asleep = false;
        drop(is_asleep);
        println!("Server is UP again");

        let mut ip_map = self.servers_info.id_to_ip.lock().await;
        let mut my_id = self.servers_info.my_id.lock().await;
        println!("my_id that will be removed: {}", *my_id);
        println!("ip map: {:?}", ip_map);
        
        let my_ip = ip_map.get(&*my_id).unwrap().clone();       // My cuurent ip
        
        let mut max_id = std::i32::MIN;
        for (key, _value) in &*ip_map {
            if key.parse::<i32>().unwrap() > max_id {
                max_id = key.parse::<i32>().unwrap();
            }
        }
        println!("max_id: {}", max_id);
        ip_map.remove(&*my_id);                 // Remove my id from the map
        
        let id_as_int = max_id;         // Convert max id to int
        let new_id = id_as_int + 1;     // Increment max id by 1
        println!("Removed myself from the map");


        ip_map.insert(new_id.to_string(), my_ip.clone());
        println!("Inserted new id: {} in the map", new_id);
        println!("ip map: {:?}", ip_map);
        let client = reqwest::Client::new();
        for (key, value) in &*ip_map {
            if key != &new_id.to_string() {
                let _request = client.get(value).header("fn", "broadcast_id")
                .header("old_id", &*my_id)
                .header("new_id", &new_id.to_string())
                .header("new_ip", &my_ip)
                .send()
                .await;
                println!("Sent broadcast_id to {}", value);
            }
            println!("Server id: {} --> IP {}", key, value);
        }
        *my_id = new_id.to_string();    // Update my id
        println!("---------------------------");
        
        
        Ok(format!("Force failure called"))
    }


    async fn broadcast_id(&self, headers: & mut [Header<'_>],requester_info:SocketAddr) -> Result<String, Box<dyn std::error::Error>> {
        println!("-------------Broadcast id called from {}:{}-------------", requester_info.ip(), requester_info.port());
        let mut old_id = String::new();
        let mut new_id = String::new();
        let mut new_ip = String::new();
        for header in headers {
            if header.name == "old_id" {
                old_id = String::from_utf8(header.value.to_vec()).unwrap();
            }
            if header.name == "new_id" {
                new_id = String::from_utf8(header.value.to_vec()).unwrap();
            }
            if header.name == "new_ip" {
                new_ip = String::from_utf8(header.value.to_vec()).unwrap();
            }
        }
        println!("old_id: {}", old_id);
        println!("new_id: {}", new_id);
        println!("new_ip: {}", new_ip);

        let mut ip_map = self.servers_info.id_to_ip.lock().await;
        ip_map.remove(&old_id);
        ip_map.insert(new_id, new_ip);

        for (key, value) in &*ip_map {
            println!("Server id: {} --> IP {}", key, value);
        }
        println!("---------------------------");
        Ok(format!("Broadcast id called"))
    }

    async fn ping(&self, headers: & mut [Header<'_>],_requester_info:SocketAddr) -> Result<String, Box<dyn std::error::Error>> {
        let id = headers.iter().find(|h| h.name == "id").unwrap().value;
        // let mut success = self.servers_info.successeful_requests.lock().await;
        // *success += 1;
        // drop(success);
        println!("Ping called {}", String::from_utf8(id.to_vec()).unwrap());
        Ok(format!("Ping called"))
    }

}




#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let ip_server1 = &args[1];
    let ip_server2 = &args[2];
    let ip_server3 = &args[3];
    
    let main_server_index = &args[4].parse::<usize>().unwrap();
    
    let servers = vec![ip_server1, ip_server2, ip_server3];
    let mut ips: Vec<String> = Vec::new();
    for i in 0..servers.len() {
        if i != *main_server_index {
            ips.push(servers[i].to_string());
        }
    }
    let server_addr:SocketAddr = servers[*main_server_index].parse().unwrap();
    let ip1 = format!("{}{}","http://" ,servers[0]);
    let ip2 = format!("{}{}","http://" ,servers[1]);
    let ip3 = format!("{}{}","http://" ,servers[2]);
    let my_id = format!("{}", main_server_index.to_string());
    println!("Main server listening on index: {}", *main_server_index);
    println!("Server1 listening on {}", ip1);
    println!("Server2 listening on {}", ip2);
    println!("Server3 listening on {}", ip3);

    println!("Main server {}", server_addr);

    // Create output file
    let path = format!("servers_results/server{}.txt", main_server_index);
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();

    let mut num_of_failed_requests = 0;
    let mut total_num_of_requests : i64 = 0;
    let listener = TcpListener::bind(server_addr).unwrap();
    
    let server_arc : Arc<Mutex<Server>> = Arc::new(Mutex::new(Server::new(ip1.to_string(), ip2.to_string(), ip3.to_string(), my_id.to_string())));

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
    
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        let requester_info = stream.peer_addr().unwrap();
        let s = server_arc.clone();
        let s_copy = server_arc.clone();
        
        total_num_of_requests += 1;
        match s_copy.try_lock() {
            Some(_)=>{
                stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
                stream.flush().unwrap();
                if total_num_of_requests % 1000 == 0 {
                    let _ = writeln!(file, "Total requests: {} Failed requests: {} Successful requests: {}",total_num_of_requests, num_of_failed_requests, total_num_of_requests - num_of_failed_requests);
                }
                let _join = task::spawn(async move{
                    handle_connection(s, buffer, requester_info).await;
                });
            }
            None => {
                println!("Server is asleep");
                num_of_failed_requests += 1;
            }
        };
        
    }
}


//cargo run  --bin server 10.40.52.93:50050 10.40.52.93:50051 10.40.52.93:50052 0 
// 10.40.52.93

async fn handle_connection(server:Arc<Mutex<Server>>, buffer: [u8; 1024], requester_info: SocketAddr) {
    
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let _res = req.parse(&buffer).unwrap();

    // println!("res: {:?}", res);
    // println!("req: {:?}", req);
    
    let mut fn_name: String = String::new();
    for header in req.headers[..].iter() {
        if header.name == "fn" {
            fn_name = std::str::from_utf8(header.value).unwrap().to_string();
            // println!("fn_name: {}", fn_name);
        }
    }
    
    // let server =Server::new(ip1.to_string(), ip2.to_string(), ip3.to_string(), my_id.to_string());
    let server = server.lock().await;
    match fn_name.as_str() {
        "init_election" => {
            let _res = server.init_election(req.headers, requester_info).await;
        },
        "force_failure" => {
            let _res = server.force_failure(req.headers, requester_info).await;
        },
        "broadcast_id" => {
            let _res = server.broadcast_id(req.headers, requester_info).await;
        },
        "ping" => {
            let _res = server.ping(req.headers, requester_info).await;
        },
        _ => {
            println!("No function found");
        }
    }
        
    

    // let get = b"GET / HTTP/1.1\r\n";
    // let sleep = b"GET /sleep HTTP/1.1\r\n";

    // let (status_line, filename) = if buffer.starts_with(get) {
    //     ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    // } else if buffer.starts_with(sleep) {
    //     thread::sleep(Duration::from_secs(5));
    //     ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    // } else {
    //     ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    // };

    // let contents = fs::read_to_string(filename).unwrap();

    // let response = format!("{}{}", status_line, contents);

    // stream.write(response.as_bytes()).unwrap();
    // stream.flush().unwrap();

}