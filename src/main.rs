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

//later on this should be switced to async (harder to debug, so do when working)
fn main() {
    //this is loading the configuration file from config.yaml
    let settings = parse_yaml();
    let num_threads: &u8 = &settings[0].parse().expect("Conversiion error on reading yaml to variable.");
    let target = &settings[1];
    let http_port = &settings[2];
    let https_port = &settings[3];

    //combining port and target ip range together for http
    let mut http_socket = target.clone().to_owned();
    http_socket.push_str(":");
    http_socket.push_str(&http_port);
    
    //combining port and target ip range together for http
    let mut https_socket = target.to_owned();
    https_socket.push_str(":");
    https_socket.push_str(&https_port);
   
    
    let listener = TcpListener::bind(http_socket).unwrap(); //this is the listener for http
    let sec_listener = TcpListener::bind(https_socket).unwrap(); //this is the listener for https

    let pool = ThreadPool::new((num_threads/2) as usize); //this is a pool that gets cloned to allow for two ports opened (and hence divided by 2)

    //these are the cloned ports that get used for http & https
    let pool_https = pool.clone();
    let pool_http = pool.clone();
    

    //this is the https connection
    thread::spawn(move || {
	for stream in sec_listener.incoming() {
	    match stream {
		Ok(stream) => {
		    pool_https.execute(move || {
			handle_connection(stream);
		    });
		}
		Err(_e) => {
		    eprintln!("failed to connect");
		}
	    }
	}
    });

    //this is the http connection
    thread::spawn(move || {
	for stream in listener.incoming() {
	    match stream {
		Ok(stream) => {
		    pool_http.execute(move || {
			handle_connection(stream);
		    });
		}
		Err(_e) => {
		    eprintln!("failed to connect");
		}
	    }
	}
    });
    
    //this prevents the main thread from ending (allow the previous ones to repeat)
    loop {
	thread::park();
    };

}

//this is where the main connection stuff is. Most other functions are referenced here.
fn handle_connection(mut stream: TcpStream)  {
    
    //reading the request
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // this next bit returns the bit after the /  in the http request

    let file = file_target(&request_line); //this is the file but after the slash that was entered
 
    let target = return_file(file.to_string());
     
    let mut full_file: String = "./www/".to_string(); 
    full_file.push_str(&target); //full_file is the file including the ./www/ bit
    println!("{full_file}");
    
    if request_line == "GET / HTTP/1.1" {
	//this is different so that the index.html is the landing page
	let status_line = "HTTP/1.1 200 OK";
        let contents = fs::read_to_string("www/index.html").unwrap();
        let length = contents.len();

        let response = format!(
            "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
        );

        stream.write_all(response.as_bytes()).unwrap();
    
    } else {
	let status_line = "HTTP/1.1 200 OK";
	let contents = fs::read_to_string(full_file).unwrap();
	let length = contents.len();

	let response = format!(
	    "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
	);
	stream.write_all(response.as_bytes()).unwrap();

	//note: there is no 404.html exception as this is handled by return_file()
	
    } 
}

fn file_target(request: &String) -> String {
    //note, this is done weirdly as we don't know the length of the file requested
    let length = request.len();
    let file_name_part_1 = &request.to_string()[5..length]; //removing start

    let file_name_part_2 = &file_name_part_1.chars().rev().collect::<String>(); //reversing

    let length  = file_name_part_2.len(); //overwriting length as new

    let file_name_part_3 = &file_name_part_2[9..length]; //removing start of reversed (so end of normal)
    
    let file_name = &file_name_part_3.chars().rev().collect::<String>(); //this reverses again for normal
    
    let file: String = file_name.to_owned(); //casting to string and not &reference

    file

}



//this returns a Vec<String> of the files that can be checked against
//the normal method returns Vec<ReadDir>, this is a set of glorified (and painful) conversions  
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

//this just checks if the file is in the list. Returns a bool, I should remove this later and just use .contains
fn is_in_vector(file_name: String, file_list: Vec<String>) -> bool {
    let content = file_list.contains(&file_name);
    //println!("{}", content);
    content
}



//this takes the input file (i.e., index.html) and then returns the file to look up (supports images, etc)
fn return_file(in_file: String) -> String {
    
    let dirs = list_dirs(); //dirs to check against 
    let extension = ".html"; //extension that gets appended later
    let full_file = &mut in_file.to_string(); //this is needed for appending to later (and so has to be mut) 

    
    if is_in_vector(in_file.clone(), dirs.clone())  == true { //here we are checking if the user enters a valid file, i.e. index.html or favicon.ico. .clone() is used as variables need reusing later
	in_file //returning it straigh back

    } else { //this is where we deal with files that should have .html appended to them
	full_file.push_str(extension); //appending .html to the file

	if is_in_vector(full_file.to_string(), dirs) == true { //if the .html appended file exists, it returns it
	    full_file.to_string() 
	} else { //not all .html appended files exist, so just returns the 404.html page
	    println!("404.html");
	    "404.html".to_string()
	}
    }

}




//this bit is for reading yaml for parsing as the configuration file

#[derive(Debug, Serialize, Deserialize)]
struct Config { //this is an index from the yaml configuration file. New entries need to be added in order when features are added
    num_threads: u16,
    target: String,
    http_port: String,
    https_port: String
}

fn parse_yaml() -> Vec<String> {

    let f = std::fs::File::open("config.yaml").expect("Could not open file."); //reading file
    let scrape_config: Config = serde_yaml::from_reader(f).expect("Could not read values."); //deserializing

    //this is reading the values to variables
    let num_threads: String = scrape_config.num_threads.to_string(); 
    let target: String = scrape_config.target;
    let http_port: String = scrape_config.http_port;
    let https_port: String = scrape_config.https_port;
    //appending to a vector that can then be passed to other functions
    let configs = vec![num_threads, target, http_port, https_port];
   
    //returning
    configs
}


