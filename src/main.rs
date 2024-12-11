//DO NOT FORGET TO GIT COMMIT!!!!

use threadpool::ThreadPool;
use std::{
    fs,
    path::Path,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
};
use serde::{Deserialize, Serialize};
use serde_yaml::{self};

fn main() {

    //this is loading the configuration file from config.yaml
    let settings = parse_yaml();
    let num_threads: &u8 = &settings[0].parse().expect("Conversiion error on reading yaml to variable.");
    //let thread_count: usize = usize::from(num_threads);
    let target = &settings[1];
    let port = &settings[2];

    let mut socket = target.to_owned();
    socket.push_str(":");
    socket.push_str(&port);
    println!("{}", socket);
    
    let listener = TcpListener::bind(socket).unwrap(); //note, well know port require sudo (which doesn't have cargo)
    let pool = ThreadPool::new(*num_threads as usize); //change this to allow more threads
    
    for stream in listener.incoming() { 
	
	let stream = stream.unwrap();

	pool.execute(move || {
	    
	    handle_connection(stream);
	    let threadnum = thread::current().id();
	    println!("connection on: {:?}", threadnum);
	});
}
}

//this is where the main connection stuff is. Most other functions are referenced here.
fn handle_connection(mut stream: TcpStream) {
    
    //reading the request
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // this next bit returns the bit after the /  in the http request

    let dirs = list_dirs(); 

    let file = file_target(&request_line); //this is the file inside www, i.e., index.html
    
    let contains = is_in_vector(file.to_string(), dirs);

    let mut full_file: String = "./www/".to_string(); 
    full_file.push_str(&file); //full_file is the file including the ./www/ bit

    
    if request_line == "GET / HTTP/1.1" {
	//this is different so that the index.html is the landing page
	let status_line = "HTTP/1.1 200 OK";
        let contents = fs::read_to_string("www/index.html").unwrap();
        let length = contents.len();

        let response = format!(
            "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
        );

        stream.write_all(response.as_bytes()).unwrap();
    
    } else if contains == true {
	println!("file found!!"); //diagnostics
	
	let status_line = "HTTP/1.1 200 OK";
	let contents = fs::read_to_string(full_file).unwrap();
	let length = contents.len();

	let response = format!(
	    "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
	);
	stream.write_all(response.as_bytes()).unwrap();
	// use elif to send other html pages
	
    } else {
	let status_line = "HTTP/1.1 404 NOT FOUND";
        let contents = fs::read_to_string("404.html").unwrap();
        let length = contents.len();

        let response = format!(
            "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
        );

        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn file_target(request: &String) -> String {
    let length = request.len();
    let file_name_part_1 = &request.to_string()[5..length]; //removing start

    let file_name_part_2 = &file_name_part_1.chars().rev().collect::<String>(); //reversing

    let length  = file_name_part_2.len();

    let file_name_part_3 = &file_name_part_2[9..length];
    let file_name = &file_name_part_3.chars().rev().collect::<String>();
    println!("{}", file_name);
    
    let mut file: String = file_name.to_owned();
    
    file.push_str(".html");
    file

}



//this returns a Vec<String> of the files that can be checked against 
fn list_dirs() -> Vec<String> {
        let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(Path::new("./www/")) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
           files.push(file_name.to_string());
                        }
                    }
                }
            }
        }
    }

    files
}

//this just checks if the file is in the list. Returns a bool
fn is_in_vector(file_name: String, file_list: Vec<String>) -> bool {
    let content = file_list.contains(&file_name);
    println!("{}", content);
    content
}


//this bit is for reading yaml for parsing as the configuration file

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    num_threads: u16,
    target: String,
    port: String
}

fn parse_yaml() -> Vec<String> {

    let f = std::fs::File::open("config.yaml").expect("Could not open file."); //reading file
    let scrape_config: Config = serde_yaml::from_reader(f).expect("Could not read values."); //deserializing

    //println!("{:?}", scrape_config);

    //let mut configs: Vec<String> = Vec::new();

    let num_threads: String = scrape_config.num_threads.to_string();
    
    let target: String = scrape_config.target;
    let port: String = scrape_config.port;

    let configs = vec![num_threads, target, port];
    //println!("{:?}", configs);

    configs
}
