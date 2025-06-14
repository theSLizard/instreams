use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub enum CommandEnum {
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
            _ => Err("Invalid string for Command Enum".to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum DestinationEnum {
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
pub enum WorkersEnum {
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

#[derive(Deserialize, Serialize)]
pub struct CommandMessage {
    pub key: String,
    pub receiver: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMessage {
    pub key: String,
    pub message: String,
}

