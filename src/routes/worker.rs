//use uuid::Uuid;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use std::str::FromStr;

use crate::services::state::{InstreamState};
use crate::models::command::{CommandEnum, WorkersEnum, DestinationEnum, 
    CommandMessage, RequestMessage, ResponseMessage};

// {"key":"7e8b0844-d5d4-4118-906a-a4012e1566df","receiver":"Worker100","command":"UpdateStatus"}
// curl --header "Content-Type: application/json" --request POST --data '{"key":"7e8b0844-d5d4-4118-906a-a4012e1566df","receiver":"Worker100","command":"UpdateStatus"} http://localhost:8082/command' --verbose
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
                    CommandEnum::UpdateStatus => {actual_command = &"UpdateStatus"; },
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
                        data.sender50.lock().unwrap().send(actual_command).expect("Send failed");
                    }
                    DestinationEnum::Worker100 => {
                        data.sender100.lock().unwrap().send(actual_command).expect("Send failed");
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

// example usage: 
// > curl localhost:8082/session_key
// {"message":"ea7a3185-b17d-475c-8be7-a6ba577c3d84"}

// > curl --header "Content-Type: application/json" --request POST http://localhost:8082/work --data '{"key":"ea7a3185-b17d-475c-8be7-a6ba577c3d84","message":"StartWorker100ms"}'
// > curl --header "Content-Type: application/json" --request POST http://localhost:8082/command --data '{"key":"ea7a3185-b17d-475c-8be7-a6ba577c3d84","receiver":"Worker100","command":"UpdateStatus"}'

/* 
// work payload
{
	"key": "7e8b0844-d5d4-4118-906a-a4012e1566df",
	"message": "StartWorker100ms"
} 

// command payload
{
    "key": "ea7a3185-b17d-475c-8be7-a6ba577c3d84",
    "receiver": "Worker100",
    "command": "UpdateStatus"
}
*/

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
                        if false == *data.worker10running.lock().unwrap() {
                            println!("{} starting !!", work_enum.to_string()); 
                            let _handle10ms = thread::spawn(move || 
                                worker10ms(work_enum.to_string(), 
                                &data.receiver10,
                                &data.worker10running));
                        } else {
                            println!("{} already running.", work_enum.to_string()); 
                        }
                    },
                    WorkersEnum::StopWorker10ms => {
                        if true == *data.worker10running.lock().unwrap() {
                            println!("{} stopping ...", work_enum.to_string());
                            data.sender10.lock().unwrap().send("Stop").expect("Send failed");
                        } else {
                            println!("{} is NOT even running. ", work_enum.to_string());
                        }          
                    },
                    WorkersEnum::StartWorker25ms => {
                        if false == *data.worker25running.lock().unwrap() {
                            println!("{} starting !!", work_enum.to_string()); 
                            let _handle25ms = thread::spawn(move || 
                                worker25ms(work_enum.to_string(), 
                                &data.receiver25,
                                &data.worker25running));
                        } else {
                            println!("{} already running.", work_enum.to_string()); 
                        }
                    },
                    WorkersEnum::StopWorker25ms => {
                        if true == *data.worker25running.lock().unwrap() {
                            println!("{} stopping ...", work_enum.to_string());
                            data.sender25.lock().unwrap().send("Stop").expect("Send failed");
                        } else {
                            println!("{} is NOT even running. ", work_enum.to_string());
                        }   
                    },
                    WorkersEnum::StartWorker50ms => {
                        if false == *data.worker50running.lock().unwrap() {
                            println!("{} starting !!", work_enum.to_string()); 
                            let _handle50ms = thread::spawn(move || 
                                worker50ms(work_enum.to_string(), 
                                &data.receiver50,
                                &data.worker50running));
                        } else {
                            println!("{} already running.", work_enum.to_string()); 
                        }
                    },
                    WorkersEnum::StopWorker50ms => {
                        if true == *data.worker50running.lock().unwrap() {
                            println!("{} stopping ...", work_enum.to_string());
                            data.sender50.lock().unwrap().send("Stop").expect("Send failed");
                        } else {
                            println!("{} is NOT even running. ", work_enum.to_string());
                        }   
                    },
                    WorkersEnum::StartWorker100ms => {
                        if false == *data.worker100running.lock().unwrap() {
                            println!("{} starting !!", work_enum.to_string()); 
                            let _handle100ms = thread::spawn(move || 
                                worker100ms(work_enum.to_string(), 
                                &data.receiver100,
                                &data.worker100running));
                        } else {
                            println!("{} already running.", work_enum.to_string()); 
                        }
                    },
                    WorkersEnum::StopWorker100ms => {
                        if true == *data.worker100running.lock().unwrap() {
                            println!("{} stopping ...", work_enum.to_string());
                            data.sender100.lock().unwrap().send("Stop").expect("Send failed");
                        } else {
                            println!("{} is NOT even running. ", work_enum.to_string());
                        }   
                    },
                    WorkersEnum::StartWorker250ms => {
                        if false == *data.worker250running.lock().unwrap() {
                            println!("{} starting !!", work_enum.to_string()); 
                            let _handle250ms = thread::spawn(move || 
                                worker250ms(work_enum.to_string(), 
                                &data.receiver250,
                                &data.worker250running));
                        } else {
                            println!("{} already running.", work_enum.to_string()); 
                        }
                    },
                    WorkersEnum::StopWorker250ms => {
                        if true == *data.worker250running.lock().unwrap() {
                            println!("{} stopping ...", work_enum.to_string());
                            data.sender250.lock().unwrap().send("Stop").expect("Send failed");
                        } else {
                            println!("{} is NOT even running. ", work_enum.to_string());
                        }   
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

/////////////////////
////// workers //////
/////////////////////
fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms))
}

fn msg_loop(identifier: String, receiver: &Mutex<mpsc::Receiver<&str>>) {

    // set doze interval
    let doze: u64 = match identifier.as_str() {
        "StartWorker10ms" => 10,
        "StartWorker25ms" => 25,
        "StartWorker50ms" => 50,
        "StartWorker100ms" => 100,
        "StartWorker250ms" => 250,
        _ => identifier.parse::<u64>().unwrap_or_default()
    };

    // common processing & message loop
    loop { 
        match receiver.lock().unwrap().try_recv() {
            Ok(msg) => {
                println!("Received by {} => {}", identifier, msg);
                let command_to_execute = CommandEnum::from_str(&msg);
                match command_to_execute { 
                    Ok(command) => {
                        match command {
                            CommandEnum::Stop => { 
                                println!("... stopping.");
                                break;
                            }, 
                            CommandEnum::Start => {
                                println!("... starting.");
                            }, 
                            CommandEnum::Restart => {
                                println!("... re-starting.");
                            },
                            CommandEnum::Terminate => {
                                println!("... terminating.");
                            },
                            CommandEnum::UpdateStatus => {
                                println!("... updating status.");
                            },
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }

            Err(mpsc::TryRecvError::Empty) => {
                // Channel is empty, do other work or sleep
                sleep_ms(doze);
            }

            Err(mpsc::TryRecvError::Disconnected) => {
                println!("Sender disconnected, exiting loop.");
                break;
            }
        }
    }
}


fn worker10ms(name: String, receiver: &Mutex<mpsc::Receiver<&str>>, running: &Mutex<bool>) {
    
    *running.lock().unwrap() = true;
    println!("Spawning thread: {}", name);

    // when this returns, the worker will exit
    msg_loop(name, receiver);

    println!("Thread exiting ..");
    *running.lock().unwrap() = false;
}

fn worker25ms(name: String, receiver: &Mutex<mpsc::Receiver<&str>>, running: &Mutex<bool>) {
    *running.lock().unwrap() = true;
    println!("Spawning thread: {}", name);

    // when this returns, the worker will exit
    msg_loop(name, receiver);

    println!("Thread exiting ..");
    *running.lock().unwrap() = false;
}

fn worker50ms(name: String, receiver: &Mutex<mpsc::Receiver<&str>>, running: &Mutex<bool>) {
    *running.lock().unwrap() = true;
    println!("Spawning thread: {}", name);

    // when this returns, the worker will exit
    msg_loop(name, receiver);

    println!("Thread exiting ..");
    *running.lock().unwrap() = false;
}

fn worker100ms(name: String, receiver: &Mutex<mpsc::Receiver<&str>>, running: &Mutex<bool>) {
    *running.lock().unwrap() = true;
    println!("Spawning thread: {}", name);

    // when this returns, the worker will exit
    msg_loop(name, receiver);

    println!("Thread exiting ..");
    *running.lock().unwrap() = false;
}

fn worker250ms(name: String, receiver: &Mutex<mpsc::Receiver<&str>>, running: &Mutex<bool>) {
    *running.lock().unwrap() = true;
    println!("Spawning thread: {}", name);

    // when this returns, the worker will exit
    msg_loop(name, receiver);

    println!("Thread exiting ..");
    *running.lock().unwrap() = false;
}
/////////////////////
////// /workers /////
/////////////////////