mod models;
mod services;
mod routes;

use std::env;
use std::fmt;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use std::net::{TcpListener, SocketAddr};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};

use crate::routes::session::{hello, status, session_key};
use crate::routes::worker::{execute, send_command, };
use crate::services::state::{InstreamState};
use crate::models::instruction::{ProgramSource};
use crate::models::command::{WorkersEnum, ResponseMessage};

// test usage: ( start executable with: "args": ["-s", "127.0.0.1:8082"] )
// or use: > cargo run -- -s (selects random port)
//
//> curl localhost:8082/session_key (obtain 'session_key')
//> curl --header "Content-Type: application/json" --request POST --data '{"key": "{session_key}", "message": "StartWorker10ms"}' localhost:8082/work
//> curl --header "Content-Type: application/json" --request POST --data '{"key": "{session_key}", "receiver": "Worker10", "command": "UpdateStatus"}' localhost:8082/command       

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

// study:
/*
 http://alanclements.org/power%20pc.html
 http://alanclements.org/arm.html
 http://alanclements.org/COA_Student_Workbook_V5.1.pdf
 https://llvm.org/docs/LangRef.html
 https://github.com/rust-lang/rust/pull/38482
 https://github.com/rust-lang/compiler-builtins/blob/master/src/int/mul.rs
*/


// uplads program to execute. Usage example:
/* curl --header "Content-Type: application/json" --request POST   
--data '{"instructions":[{"opcode": "add","imdval": "0x","regsrc": 1,"regext": 0,"regdst": 2},
{"opcode": "sub","imdval": "0x","regsrc": 3,"regext": 0,"regdst": 4}]}' http://localhost:8081/load --verbose */

/* example instructions

{
    "instructions": [
        {
            "opcode": "add",
            "imdval": "0x",
            "regsrc": 1,
            "regext": 0,
            "regdst": 2
        },
        {
            "opcode": "sub",
            "imdval": "0x",
            "regsrc": 3,
            "regext": 0,
            "regdst": 4
        }
    ]
}

*/

// copy-paste: curl --header "Content-Type: application/json" --request POST --data '{"instructions":[{"opcode": "add","imdval": "0x","regsrc": 1,"regext": 0,"regdst": 2}, {"opcode": "sub","imdval": "0x","regsrc": 3,"regext": 0,"regdst": 4}]}' http://localhost:8082/load --verbose
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
                    let (sender50, receiver50_orig) = mpsc::channel();
                    let (sender100, receiver100_orig) = mpsc::channel();
                    let (sender250, receiver250_orig) = mpsc::channel();

                    // Clone the senders for each thread
                    let sender10_clone = sender10.clone();
                    let sender25_clone = sender25.clone();
                    let sender50_clone = sender50.clone();
                    let sender100_clone = sender100.clone();
                    let sender250_clone = sender250.clone();

                    let stream_state = web::Data::new(Arc::new(InstreamState {
                        
                        
                        // these could be replaced by ..Default::default()
                        master_key: Mutex::new(0.to_string()),
                        code_segment: Vec::new().into(),

                        worker10running: Mutex::new(false),
                        worker25running: Mutex::new(false),
                        worker50running: Mutex::new(false),
                        worker100running: Mutex::new(false),
                        worker250running: Mutex::new(false),
                        

                        sender10: sender10_clone.into(),
                        receiver10: receiver10_orig.into(), // Mutex::new(receiver10_orig),
                        sender25: sender25_clone.into(),
                        receiver25: receiver25_orig.into(), // Mutex::new(receiver25_orig),
                        sender50: sender50_clone.into(),
                        receiver50: receiver50_orig.into(), // Mutex::new(receiver50_orig),
                        sender100: sender100_clone.into(),
                        receiver100: receiver100_orig.into(), // Mutex::new(receiver100_orig),
                        sender250: sender250_clone.into(),
                        receiver250: receiver250_orig.into(), // Mutex::new(receiver250_orig),
                    
                        //..Default::default()
                    
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
