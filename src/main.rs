
use std::env;
use std::sync::{Arc, Mutex};

use std::net::{TcpListener, SocketAddr};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

struct InstreamState {
    master_key: Mutex<String>,
    code_segment: Mutex<Vec<Instruction>>, 
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

                    let stream_state = web::Data::new(Arc::new(InstreamState {
                        master_key: Mutex::new(0.to_string()),
                        code_segment: Vec::new().into(),
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
