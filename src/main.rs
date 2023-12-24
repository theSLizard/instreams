
use std::env;
use std::fmt;
use std::str::FromStr;

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

use std::net::{TcpListener, SocketAddr};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// test usage: ( start executable with: "args": ["-s", "127.0.0.1:8082"] )
// or use: > cargo run -- -s (selects random port)
//
//> curl localhost:8082/session_key (obtain 'session_key')
//> curl --header "Content-Type: application/json" --request POST --data '{"key": "{session_key}", "message": "StartWorker10ms"}' localhost:8082/work
//> curl --header "Content-Type: application/json" --request POST --data '{"key": "{session_key}", "receiver": "Worker10", "command": "UpdateStatus"}' localhost:8082/command       

struct InstreamState {
    master_key: Mutex<String>,
    code_segment: Mutex<Vec<Instruction>>, 
    sender10: Mutex<mpsc::Sender<&'static str>>, 
    sender25: Mutex<mpsc::Sender<&'static str>>, 
    sender250: Mutex<mpsc::Sender<&'static str>>, 
    receiver10: Mutex<mpsc::Receiver<&'static str>>, 
    receiver25: Mutex<mpsc::Receiver<&'static str>>,
    receiver250: Mutex<mpsc::Receiver<&'static str>>,

}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseMessage {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestMessage {
    key: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandMessage {
    key: String,
    receiver: String,
    command: String,
}

#[derive(Debug, Deserialize)]
enum CommandEnum {
    Stop,
    Start,
    Restart,
    Terminate,
    UpdateStatus,
}

impl FromStr for CommandEnum {

    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        match s {
            "Stop" => Ok(CommandEnum::Stop),
            "Start" => Ok(CommandEnum::Start),
            "Restart" => Ok(CommandEnum::Restart),
            "Terminate" => Ok(CommandEnum::Terminate),
            "UpdateStatus" => Ok(CommandEnum::UpdateStatus),
            _ => Err("Invalid string for Comman dEnum".to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
enum DestinationEnum {
    Worker10,
    Worker25,
    Worker50,
    Worker100,
    Worker250,
}

impl FromStr for DestinationEnum {

    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        match s {
            "Worker10" => Ok(DestinationEnum::Worker10),
            "Worker25" => Ok(DestinationEnum::Worker25),
            "Worker50" => Ok(DestinationEnum::Worker50),
            "Worker100" => Ok(DestinationEnum::Worker100),
            "Worker250" => Ok(DestinationEnum::Worker250),
            _ => Err("Invalid string for Destination Enum".to_string()),
        }
    }
}


#[derive(Debug, Deserialize)]
enum WorkersEnum {
    StartWorker10ms,
    StartWorker25ms,
    StartWorker50ms,
    StartWorker100ms,
    StartWorker250ms,
    StopWorker10ms,
    StopWorker25ms,
    StopWorker50ms,
    StopWorker100ms,
    StopWorker250ms,
}

impl FromStr for WorkersEnum {

    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        match s {

            "StartWorker10ms" => Ok(WorkersEnum::StartWorker10ms),
            "StartWorker25ms" => Ok(WorkersEnum::StartWorker25ms),
            "StartWorker50ms" => Ok(WorkersEnum::StartWorker50ms),
            "StartWorker100ms" => Ok(WorkersEnum::StartWorker100ms),
            "StartWorker250ms" => Ok(WorkersEnum::StartWorker250ms),

            "StopWorker10ms" => Ok(WorkersEnum::StopWorker10ms),
            "StopWorker25ms" => Ok(WorkersEnum::StopWorker25ms),
            "StopWorker50ms" => Ok(WorkersEnum::StopWorker50ms),
            "StopWorker100ms" => Ok(WorkersEnum::StopWorker100ms),
            "StopWorker250ms" => Ok(WorkersEnum::StopWorker250ms),

            _ => Err("Invalid string for WorkersEnum".to_string()),
        }
    }
}

impl fmt::Display for WorkersEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        // Implement how each variant should be displayed
        match self {

            WorkersEnum::StartWorker10ms => write!(f, "StartWorker10ms"),
            WorkersEnum::StartWorker25ms => write!(f, "StartWorker25ms"),
            WorkersEnum::StartWorker50ms => write!(f, "StartWorker50ms"),
            WorkersEnum::StartWorker100ms => write!(f, "StartWorker100ms"),
            WorkersEnum::StartWorker250ms => write!(f, "StartWorker250ms"),

            WorkersEnum::StopWorker10ms => write!(f, "StopWorker10ms"),
            WorkersEnum::StopWorker25ms => write!(f, "StopWorker25ms"),
            WorkersEnum::StopWorker50ms => write!(f, "StopWorker50ms"),
            WorkersEnum::StopWorker100ms => write!(f, "StopWorker100ms"),
            WorkersEnum::StopWorker250ms => write!(f, "StopWorker250ms"),
            
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Instruction {
    opcode: String,
    imdval: String,
    regsrc: u8,
    regext: u8, 
    regdst: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProgramSource {
    instructions: Vec<Instruction>,
}

/////////////////////
////// workers //////
/////////////////////
fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms))
}


fn worker10ms(name: String, receiver: &Mutex<mpsc::Receiver<&str>>) {
    
    println!("Spawning thread: {}", name);
    
    loop { 
        match receiver.lock().unwrap().recv()  { // szlo1: todo: this needs to NOT be blocking !! #get_back_to_this
            Ok(message) => { // szlo1: message passing logic OK otherwise 
                println!("Received by {} => {}", name, message);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    
        sleep_ms(10);
        //println!("|| --> 10 ms"); 
    }
}

fn worker25ms(name: String, receiver: &Mutex<mpsc::Receiver<&str>>) {
    loop {
        sleep_ms(25);
        println!("|| --> 25 ms"); 
    }
}

fn worker250ms(name: String, receiver: &Mutex<mpsc::Receiver<&str>>) {
    loop {
        sleep_ms(250);
        println!("|| --> 250 ms"); 
    }
}
/////////////////////
////// /workers /////
/////////////////////

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json(ResponseMessage {
        message: "::: instreams says HI !!".to_owned(),
    })
}

#[get("/session_key")]
async fn session_key(data: web::Data<Arc<InstreamState>>) -> impl Responder {

    let mut master_key = data.master_key.lock().unwrap();

    if *master_key == "0".to_string() {
        *master_key = Uuid::new_v4().to_string();
    }

    HttpResponse::Ok().json(ResponseMessage {
        message: master_key.to_owned(),
    })
}

#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok().json(ResponseMessage {
        message: "::: instreams is listening.".to_owned(),
    })
}

// uplads program to execute. Usage example:
/* curl --header "Content-Type: application/json" --request POST   
--data '{"instructions":[{"opcode": "add","imdval": "0x","regsrc": 1,"regext": 0,"regdst": 2},
{"opcode": "sub","imdval": "0x","regsrc": 3,"regext": 0,"regdst": 4}]}' http://localhost:8081/load --verbose */
#[post("/load")]
async fn load_program(payload: web::Json<ProgramSource>, 
                _req:HttpRequest, data: web::Data<Arc<InstreamState>>) -> impl Responder {

                    let mut instructions = data.code_segment.lock().unwrap();

                    // move instructions to the shared area
                    *instructions = payload.instructions.iter().cloned().collect();

                    return HttpResponse::Ok().json(ResponseMessage {
                        message: "Program Loaded.".to_string(),
                    })
}

// lists the program currently loaded in memory
//
// usage example:
// > curl http://localhost:8081/list 
// > {"instructions":[{"opcode":"add","imdval":"0x","regsrc":1,"regext":0,"regdst":2},{"opcode":"sub","imdval":"0x","regsrc":3,"regext":0,"regdst":4}]}
#[get("/list")]
async fn list_program(data: web::Data<Arc<InstreamState>>) -> impl Responder {

                    let instructions = data.code_segment.lock().unwrap();

                    return HttpResponse::Ok().json(ProgramSource {
                        instructions: instructions.iter().cloned().collect(),
                    })
}


/*
fn thread_function<'a>(shared_string: &'a Arc<Mutex<String>>, sender: mpsc::Sender<&'a str>) {
    // Access the shared string inside the Mutex
    let locked_string = shared_string.lock().unwrap();

    // Send the string slice through the channel
    sender.send(&*locked_string).expect("Send failed");
}
*/

#[post("/command")]
async fn send_command(payload: web::Json<CommandMessage>, 
                _req:HttpRequest, data: web::Data<Arc<InstreamState>>) -> impl Responder {

    let key = &payload.key;
    let destination = &payload.receiver;
    let command = &payload.command;

    println!("key  : {}" , key);
    println!("command : {}" , command);
    println!("destination : {}" , destination);

    let mut actual_command = &"_";
    let mut ret_value: String = "::: executing: ".to_string();
    ret_value.push_str(&command);

    if *key == data.master_key.lock().unwrap().to_string() { 

        // todo: implement sending commands to specific threads here.

        // map command to enum
        let command_to_execute = CommandEnum::from_str(&command);
        match command_to_execute {
            Ok(command) => {
                match command {
                    // todo: check how to move these strings into a resource that is linked into the data segment.
                    // so we can simply reference these string resources - is this even possible, btw ?? (it should be)
                    CommandEnum::Stop => {actual_command = &"Stop"; }, 
                    CommandEnum::Start => {actual_command = &"Start"; }, 
                    CommandEnum::Restart => {actual_command = &"Restart"; },
                    CommandEnum::Terminate => {actual_command = &"Terminate"; },
                    CommandEnum::UpdateStatus => {actual_command = &"Update_Status"; },
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                ret_value.push_str(" ::: Error :: ");
                ret_value.push_str(&e);
            }
        }


        let destination_of_command = DestinationEnum::from_str(&destination);
        match destination_of_command {
            
            Ok(destination) => {

                match destination {

                    DestinationEnum::Worker10 => {
                        data.sender10.lock().unwrap().send(actual_command).expect("Send failed");
                    }
                    DestinationEnum::Worker25 => {
                        data.sender25.lock().unwrap().send(actual_command).expect("Send failed");
                    }
                    DestinationEnum::Worker50 => {
                        //
                    }
                    DestinationEnum::Worker100 => {
                        //
                    }
                    DestinationEnum::Worker250 => {
                        data.sender250.lock().unwrap().send(actual_command).expect("Send failed");
                    }

                }

            }

            Err(e) => {
                println!("Error: {}", e);
                ret_value.push_str(" ::: Error :: ");
                ret_value.push_str(&e);
            }
        }

        return HttpResponse::Ok().json(ResponseMessage {
            message: ret_value,
        })
    } else {
        return HttpResponse::Forbidden().json(ResponseMessage {
            message: "::. Key? ..".to_owned(),
        });
    }
}



#[post("/work")]
async fn execute(payload: web::Json<RequestMessage>, 
                _req:HttpRequest, data: web::Data<Arc<InstreamState>>) -> impl Responder {

    let key = &payload.key;
    let work: &String = &payload.message;

    println!("key  : {}" , key);
    println!("work : {}" , work);

    let mut ret_value: String = "::: executing: ".to_string();
    ret_value.push_str(&work);

    if *key == data.master_key.lock().unwrap().to_string() { 

        let mut json_match: String = "WorkersEnum::".to_string(); json_match.push_str(&work);
        // note: from_str is part of then enum's custom implementation !!
        let work_to_do = WorkersEnum::from_str(&work);
    
        match work_to_do {
            
            Ok(work_enum) => {

                // Do something with the enum
                match work_enum {
                    WorkersEnum::StartWorker10ms => { 
                        println!("{} starting ...", work_enum.to_string()); 
                        let _handle10ms = thread::spawn(move || worker10ms(work_enum.to_string(), &data.receiver10));

                    },
                    WorkersEnum::StopWorker10ms => {
                        println!("{} starting ...", work_enum.to_string()); 
                    },
                    WorkersEnum::StartWorker25ms => {
                        println!("{} starting ...", work_enum.to_string());
                        let _handle25ms = thread::spawn(move || worker25ms(work_enum.to_string(), &data.receiver25));
                    },
                    WorkersEnum::StopWorker25ms => {
                        println!("{} starting ...", work_enum.to_string()); 
                    },
                    WorkersEnum::StartWorker50ms => {
                        println!("{} starting ...", work_enum.to_string()); 
                    },
                    WorkersEnum::StopWorker50ms => {
                        println!("{} starting ...", work_enum.to_string()); 
                    },
                    WorkersEnum::StartWorker100ms => {
                        println!("{} starting ...", work_enum.to_string()); 
                    },
                    WorkersEnum::StopWorker100ms => {
                        println!("{} starting ...", work_enum.to_string()); 
                    },
                    WorkersEnum::StartWorker250ms => {
                        println!("{} starting ...", work_enum.to_string()); 

                        let _handle250ms = thread::spawn(move || worker250ms(work_enum.to_string(), &data.receiver250));
                    },
                    WorkersEnum::StopWorker250ms => {
                        println!("{} starting ...", work_enum.to_string()); 
                    },
                    
                }
            }
            
            Err(e) => {
                println!("Error: {}", e);
                ret_value.push_str(" ::: Error :: ");
                ret_value.push_str(&e);
            }
        }

        return HttpResponse::Ok().json(ResponseMessage {
            message: ret_value,
        })

    
    } else {
        return HttpResponse::Forbidden().json(ResponseMessage {
            message: "::. Key? ..".to_owned(),
        });
    }
    
}
                

// run server with 
// > ./instreams -s 
// (uses defaults + random port)
// or 
// > ./instreams -s 127.0.0.1:8081 
// (specify ip address and port from command line)
#[actix_rt::main]
async fn main() -> std::io::Result<()>{

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        for arg in &args[1..] {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!("--- you're on your own !!");
                    return Ok(());
                },
                "-v" | "--version" => {
                    println!("--- instreams 0.1");
                    return Ok(());
                },
                "-s" => {

                    println!(":.:.:");

                    let socket_address = if args.len() > 2 { args[2].to_string() } 
                                                        else { "127.0.0.1:0".to_string() };


                     // Create a channels
                    let (sender10, receiver10_orig) = mpsc::channel();
                    let (sender25, receiver25_orig) = mpsc::channel();
                    let (sender250, receiver250_orig) = mpsc::channel();


                    // Clone the senders for each thread
                    let sender10_clone = sender10.clone();
                    let sender25_clone = sender25.clone();
                    let sender250_clone = sender250.clone();

                    let stream_state = web::Data::new(Arc::new(InstreamState {
                        master_key: Mutex::new(0.to_string()),
                        code_segment: Vec::new().into(),
                        sender10: sender10_clone.into(),
                        receiver10: receiver10_orig.into(), // Mutex::new(receiver10_orig),
                        sender25: sender25_clone.into(),
                        receiver25: receiver25_orig.into(), // Mutex::new(receiver25_orig),
                        sender250: sender250_clone.into(),
                        receiver250: receiver250_orig.into(), // Mutex::new(receiver250_orig),
                    }));

                    let tcp_listener = match TcpListener::bind(socket_address) {
                            Ok(listener) => listener,
                            Err(error) => {
                                eprintln!("::: Failed to bind to address: {}", error);
                                return Ok(()) // Err(error);
                            }
                        };
                    

                    let socket_address: SocketAddr = match tcp_listener.local_addr() {
                        Ok(address) => address,
                        Err(error) => {
                            eprintln!("::: {}", error);
                            return Ok(()) //Err(error);
                        }
                    };

                    println!(":: addr {:?}", socket_address.ip());
                    println!(":: port {:?}", socket_address.port());

                    let server = HttpServer::new(move || App::new()
                                                        .app_data(stream_state.clone())
                                                        .service(hello)
                                                        .service(status)
                                                        .service(execute)
                                                        .service(send_command)
                                                        .service(session_key)
                                                        .service(load_program)
                                                        .service(list_program))
                                                        .listen(tcp_listener)?;
                    let _ = server.run()
                    .await;
                    return Ok(());

                },
                _ => {
                    println!(".::.");
                    return Ok(());
                }
            }
        }
    }

    return Ok(());
}
