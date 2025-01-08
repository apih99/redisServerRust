use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct Value {
    data: String,
    expiry: Option<SystemTime>,
}

impl Value {
    pub fn new(data: String, expiry: Option<Duration>) -> Self {
        let expiry = expiry.map(|duration| SystemTime::now() + duration);
        Self { data, expiry }
    }

    pub fn is_expired(&self) -> bool {
        self.expiry
            .map(|expiry| SystemTime::now() > expiry)
            .unwrap_or(false)
    }

    pub fn parse_int(&self) -> Option<i64> {
        self.data.parse::<i64>().ok()
    }

    // Helper method to get remaining duration
    fn remaining_duration(&self) -> Option<Duration> {
        self.expiry.and_then(|expiry| {
            expiry.duration_since(SystemTime::now()).ok()
        })
    }
}

#[derive(Clone, Default)]
pub struct Store {
    data: Arc<Mutex<HashMap<String, Value>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: String, value: String, expiry: Option<Duration>) {
        let mut data = self.data.lock().unwrap();
        data.insert(key, Value::new(value, expiry));
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut data = self.data.lock().unwrap();
        if let Some(value) = data.get(key) {
            if value.is_expired() {
                data.remove(key);
                None
            } else {
                Some(value.data.clone())
            }
        } else {
            None
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        let mut data = self.data.lock().unwrap();
        if let Some(value) = data.get(key) {
            if value.is_expired() {
                data.remove(key);
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn del(&self, keys: &[String]) -> i64 {
        let mut data = self.data.lock().unwrap();
        let mut deleted = 0;
        for key in keys {
            if let Some(value) = data.get(key) {
                if !value.is_expired() {
                    data.remove(key);
                    deleted += 1;
                }
            }
        }
        deleted
    }

    pub fn incr(&self, key: &str) -> Result<i64, &'static str> {
        let mut data = self.data.lock().unwrap();
        
        if let Some(value) = data.get(key) {
            if value.is_expired() {
                data.remove(key);
                let new_value = Value::new("1".to_string(), None);
                data.insert(key.to_string(), new_value);
                Ok(1)
            } else {
                match value.parse_int() {
                    Some(n) => {
                        let new_n = n + 1;
                        // Preserve the original expiry by converting it to a duration
                        let expiry = value.remaining_duration();
                        let new_value = Value::new(new_n.to_string(), expiry);
                        data.insert(key.to_string(), new_value);
                        Ok(new_n)
                    }
                    None => Err("ERR value is not an integer or out of range"),
                }
            }
        } else {
            let new_value = Value::new("1".to_string(), None);
            data.insert(key.to_string(), new_value);
            Ok(1)
        }
    }

    pub fn decr(&self, key: &str) -> Result<i64, &'static str> {
        let mut data = self.data.lock().unwrap();
        
        if let Some(value) = data.get(key) {
            if value.is_expired() {
                data.remove(key);
                let new_value = Value::new("-1".to_string(), None);
                data.insert(key.to_string(), new_value);
                Ok(-1)
            } else {
                match value.parse_int() {
                    Some(n) => {
                        let new_n = n - 1;
                        // Preserve the original expiry by converting it to a duration
                        let expiry = value.remaining_duration();
                        let new_value = Value::new(new_n.to_string(), expiry);
                        data.insert(key.to_string(), new_value);
                        Ok(new_n)
                    }
                    None => Err("ERR value is not an integer or out of range"),
                }
            }
        } else {
            let new_value = Value::new("-1".to_string(), None);
            data.insert(key.to_string(), new_value);
            Ok(-1)
        }
    }
} 