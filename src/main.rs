mod thread_pool;
use httparse::Header;

use std::collections::HashMap;
use std::env;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::net::TcpListener;

use std::fs;
use std::sync::Arc;

use std::thread;
use std::thread::sleep;
use std::time::Duration;

use httparse;

use tokio::task;
// use tokio::sync::Mutex;
use futures::lock::Mutex;

struct ServersInfo {
    id_to_ip: Arc<Mutex<HashMap<String, String>>>,
    pub my_id : Arc<Mutex<String>>,
}

impl ServersInfo {
    fn new(_ip1:String, _ip2:String, _ip3:String, _my_id:String) -> Self {

        let mut ip_id: HashMap<String, String> = HashMap::new();
        
        ip_id.insert("0".to_string(), _ip1.clone());
        ip_id.insert("1".to_string(), _ip2.clone());
        ip_id.insert("2".to_string(), _ip3.clone());
    
        // println!("------------Constructor-----------");
        // for (key, value) in &ip_id {
        //     println!("Server id: {} --> IP {}", key, value);
        // }
        
        Self {
            id_to_ip: Arc::new(Mutex::new(ip_id)),
            my_id: Arc::new(Mutex::new(_my_id)),
        }
    }
}
struct Server {
    servers_info: ServersInfo
}
async fn send_request() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let request = client.get("http://10.65.192.142:50051").header("fn", "force_failure")
    .send()
    .await?
    .text()
    .await?;
    // let body = client.get("http://127.0.0.1:7878").send()

    Ok(request)
}

impl Server {
    fn new(_ip1:String, _ip2:String, _ip3:String, _my_id:String) -> Self {
        Self {
            servers_info: ServersInfo::new(_ip1, _ip2, _ip3, _my_id)
        }
    }

    async fn init_election(&self, headers: & mut [Header<'_>],requester_info:SocketAddr) -> Result<String, Box<dyn std::error::Error>> {
        println!("-------------Init election called from {}:{}-------------", requester_info.ip(), requester_info.port());
        let mut id = String::new();
        for header in headers {
            if header.name == "id" {
                id = String::from_utf8(header.value.to_vec()).unwrap();
            }
        }
        println!("id: {}", id);

        let mut min_id = std::i32::MAX;
        let map = self.servers_info.id_to_ip.lock().await;

        for (key, _value) in &*map {
            if key.parse::<i32>().unwrap() < min_id {
                min_id = key.parse::<i32>().unwrap();
            }
        }
        println!("min_id: {}", min_id);

        match map.get(&min_id.to_string()){
            Some(ip) => {
                println!("Sending request to: {}", ip);
                let client = reqwest::Client::new();
                let _request = client.get("http://10.65.192.142:50050")
                .header("fn", "force_failure")
                .send().await;
               
            },
            None => {
                println!("Couldn't find min id in Map");
            }
        }

        

        let _res = send_request().await;
        println!("---------------------------");
        
        Ok(format!("min_id: {}", min_id))

    }

    async fn force_failure(&self, headers: & mut [Header<'_>],requester_info:SocketAddr) -> Result<String, Box<dyn std::error::Error>> {
        println!("-------------Force failure called from {}:{}-------------", requester_info.ip(), requester_info.port());
        sleep(Duration::from_secs(5));
        println!("Server is UP again");

        let mut ip_map = self.servers_info.id_to_ip.lock().await;
        let my_id = self.servers_info.my_id.lock().await;
        
        let my_ip = ip_map.get(&*my_id).unwrap().clone();       // My cuurent ip
        let max_id = ip_map.keys().max().unwrap().clone();      // Max id in the network
        ip_map.remove(&*my_id);                 // Remove my id from the map
        
        let id_as_int = max_id.parse::<i32>().unwrap();        // Convert max id to int
        let new_id = id_as_int + 1;     // Increment max id by 1
        println!("Removed myself from the map");


        ip_map.insert(new_id.to_string(), my_ip.clone());
        println!("Inserted new id in the map");
        
        for (key, value) in &*ip_map {
            if key != &*my_id {
                let client = reqwest::Client::new();
                let _request = client.get(value).header("fn", "broadcast_id")
                .header("old_id", &*my_id)
                .header("new_id", &new_id.to_string())
                .header("new_ip", &my_ip)
                .send()
                .await?
                .text()
                .await?;
            }
            println!("Server id: {} --> IP {}", key, value);
        }
        
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

    async fn ping(&self, headers: & mut [Header<'_>],requester_info:SocketAddr) -> Result<String, Box<dyn std::error::Error>> {
        println!("Ping called");
        let client = reqwest::Client::new();
        let _request = client.get(requester_info.ip().to_string()).header("fn", "force_failure")
        .send()
        .await?
        .text()
        .await?;
        println!("ACK!");
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

    // let _servers_info = ServersInfo::new(ip1, ip2, ip3, main_server_index.to_string());
    let ip1_ref = Arc::new(Mutex::new(ip1));
    let ip2_ref = Arc::new(Mutex::new(ip2));
    let ip3_ref = Arc::new(Mutex::new(ip3));
    let my_id_ref = Arc::new(Mutex::new(my_id));
    println!("Main server {}", server_addr);
    let listener = TcpListener::bind(server_addr).unwrap();
    
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        //print the incoming request ip  address
        let ip1_ref = Arc::clone(&ip1_ref);
        let ip2_ref = Arc::clone(&ip2_ref);
        let ip3_ref = Arc::clone(&ip3_ref);
        let my_id_ref = Arc::clone(&my_id_ref);
        let join = task::spawn(async{
            handle_connection(stream, ip1_ref, ip2_ref, ip3_ref, my_id_ref).await;
        });

    }
}


async fn handle_connection(mut stream: TcpStream, ip1: Arc<Mutex<String>>, ip2:Arc<Mutex<String>>, ip3:Arc<Mutex<String>>, my_id:Arc<Mutex<String>>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = req.parse(&buffer).unwrap();

    // println!("res: {:?}", res);
    // println!("req: {:?}", req);
    
    let mut fn_name: String = String::new();
    for header in req.headers[..].iter() {
        if header.name == "fn" {
            fn_name = std::str::from_utf8(header.value).unwrap().to_string();
            // println!("fn_name: {}", fn_name);
        }
    }
    let ip1 = ip1.lock().await;
    let ip2 = ip2.lock().await;
    let ip3 = ip3.lock().await;
    let my_id = my_id.lock().await;
    let requester_info = stream.peer_addr().unwrap();
    
    let server =Server::new(ip1.to_string(), ip2.to_string(), ip3.to_string(), my_id.to_string());
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
        
    

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

}