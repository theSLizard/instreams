use std::sync::{Mutex};
use std::sync::mpsc;

use crate::models::instruction::{Instruction};

// #[derive(Default)]
pub struct InstreamState {

    pub master_key: Mutex<String>,
    pub code_segment: Mutex<Vec<Instruction>>, 

    pub worker10running: Mutex<bool>,
    pub worker25running: Mutex<bool>,
    pub worker50running: Mutex<bool>,
    pub worker100running: Mutex<bool>,
    pub worker250running: Mutex<bool>,
    
    pub sender10: Mutex<mpsc::Sender<&'static str>>, 
    pub sender25: Mutex<mpsc::Sender<&'static str>>, 
    pub sender50: Mutex<mpsc::Sender<&'static str>>, 
    pub sender100: Mutex<mpsc::Sender<&'static str>>, 
    pub sender250: Mutex<mpsc::Sender<&'static str>>, 
    pub receiver10: Mutex<mpsc::Receiver<&'static str>>, 
    pub receiver25: Mutex<mpsc::Receiver<&'static str>>,
    pub receiver50: Mutex<mpsc::Receiver<&'static str>>,
    pub receiver100: Mutex<mpsc::Receiver<&'static str>>,
    pub receiver250: Mutex<mpsc::Receiver<&'static str>>,
}