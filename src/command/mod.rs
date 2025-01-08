use crate::resp::RespType;
use crate::store::Store;
use std::time::Duration;

#[derive(Debug)]
pub enum Command {
    Ping,
    Echo(String),
    Set {
        key: String,
        value: String,
        expiry: Option<Duration>,
    },
    Get(String),
    Exists(String),
    Del(Vec<String>),
    Incr(String),
    Decr(String),
    Unknown(String),
}

impl Command {
    pub fn from_resp(resp: RespType) -> Option<Command> {
        match resp {
            RespType::Array(Some(array)) => {
                if array.is_empty() {
                    return None;
                }

                // Get the command name
                let command_name = match &array[0] {
                    RespType::BulkString(Some(s)) => s.to_uppercase(),
                    _ => return None,
                };

                match command_name.as_str() {
                    "PING" => Some(Command::Ping),
                    "ECHO" => {
                        if array.len() != 2 {
                            return None;
                        }
                        match &array[1] {
                            RespType::BulkString(Some(s)) => Some(Command::Echo(s.clone())),
                            _ => None,
                        }
                    }
                    "SET" => {
                        if array.len() < 3 {
                            return None;
                        }
                        let key = match &array[1] {
                            RespType::BulkString(Some(s)) => s.clone(),
                            _ => return None,
                        };
                        let value = match &array[2] {
                            RespType::BulkString(Some(s)) => s.clone(),
                            _ => return None,
                        };
                        
                        let mut expiry = None;
                        if array.len() >= 5 {
                            match (&array[3], &array[4]) {
                                (RespType::BulkString(Some(opt)), RespType::BulkString(Some(val))) => {
                                    match opt.to_uppercase().as_str() {
                                        "EX" => {
                                            if let Ok(secs) = val.parse::<u64>() {
                                                expiry = Some(Duration::from_secs(secs));
                                            }
                                        }
                                        "PX" => {
                                            if let Ok(millis) = val.parse::<u64>() {
                                                expiry = Some(Duration::from_millis(millis));
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                        
                        Some(Command::Set { key, value, expiry })
                    }
                    "GET" => {
                        if array.len() != 2 {
                            return None;
                        }
                        match &array[1] {
                            RespType::BulkString(Some(s)) => Some(Command::Get(s.clone())),
                            _ => None,
                        }
                    }
                    "EXISTS" => {
                        if array.len() != 2 {
                            return None;
                        }
                        match &array[1] {
                            RespType::BulkString(Some(s)) => Some(Command::Exists(s.clone())),
                            _ => None,
                        }
                    }
                    "DEL" => {
                        if array.len() < 2 {
                            return None;
                        }
                        let mut keys = Vec::new();
                        for key in array.iter().skip(1) {
                            match key {
                                RespType::BulkString(Some(s)) => keys.push(s.clone()),
                                _ => return None,
                            }
                        }
                        Some(Command::Del(keys))
                    }
                    "INCR" => {
                        if array.len() != 2 {
                            return None;
                        }
                        match &array[1] {
                            RespType::BulkString(Some(s)) => Some(Command::Incr(s.clone())),
                            _ => None,
                        }
                    }
                    "DECR" => {
                        if array.len() != 2 {
                            return None;
                        }
                        match &array[1] {
                            RespType::BulkString(Some(s)) => Some(Command::Decr(s.clone())),
                            _ => None,
                        }
                    }
                    cmd => Some(Command::Unknown(cmd.to_string())),
                }
            }
            _ => None,
        }
    }

    pub fn execute(&self, store: &Store) -> RespType {
        match self {
            Command::Ping => RespType::SimpleString("PONG".to_string()),
            Command::Echo(msg) => RespType::BulkString(Some(msg.clone())),
            Command::Set { key, value, expiry } => {
                store.set(key.clone(), value.clone(), *expiry);
                RespType::SimpleString("OK".to_string())
            }
            Command::Get(key) => {
                match store.get(key) {
                    Some(value) => RespType::BulkString(Some(value)),
                    None => RespType::BulkString(None),
                }
            }
            Command::Exists(key) => {
                RespType::Integer(if store.exists(key) { 1 } else { 0 })
            }
            Command::Del(keys) => {
                RespType::Integer(store.del(keys))
            }
            Command::Incr(key) => {
                match store.incr(key) {
                    Ok(n) => RespType::Integer(n),
                    Err(e) => RespType::Error(e.to_string()),
                }
            }
            Command::Decr(key) => {
                match store.decr(key) {
                    Ok(n) => RespType::Integer(n),
                    Err(e) => RespType::Error(e.to_string()),
                }
            }
            Command::Unknown(cmd) => RespType::Error(format!("ERR unknown command '{}'", cmd)),
        }
    }
} 