use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instruction {
    opcode: String,
    imdval: String,
    regsrc: u8,
    regext: u8, 
    regdst: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgramSource {
    pub instructions: Vec<Instruction>,
}

