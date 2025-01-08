use bytes::{Buf, BytesMut};
use thiserror::Error;
use std::str;

#[derive(Debug, Clone, PartialEq)]
pub enum RespType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Option<Vec<RespType>>),
}

#[derive(Error, Debug)]
pub enum RespError {
    #[error("Invalid RESP data")]
    InvalidData,
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
}

impl RespType {
    pub fn parse(input: &mut BytesMut) -> Result<Option<RespType>, RespError> {
        if input.is_empty() {
            return Ok(None);
        }

        // Look for a complete command
        if !input.windows(2).any(|w| w == b"\r\n") {
            return Ok(None);
        }

        match input[0] as char {
            '+' => parse_simple_string(input),
            '-' => parse_error(input),
            ':' => parse_integer(input),
            '$' => parse_bulk_string(input),
            '*' => parse_array(input),
            _ => {
                // Handle plain text commands (redis-cli without raw mode)
                if let Some(end) = find_crlf(input) {
                    let line = str::from_utf8(&input[..end])?;
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.is_empty() {
                        return Err(RespError::InvalidData);
                    }

                    let mut array = Vec::new();
                    for part in parts {
                        array.push(RespType::BulkString(Some(part.to_string())));
                    }

                    input.advance(end + 2); // Skip CRLF
                    Ok(Some(RespType::Array(Some(array))))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            RespType::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
            RespType::Error(msg) => format!("-{}\r\n", msg).into_bytes(),
            RespType::Integer(n) => format!(":{}\r\n", n).into_bytes(),
            RespType::BulkString(None) => "$-1\r\n".to_string().into_bytes(),
            RespType::BulkString(Some(s)) => format!("${}\r\n{}\r\n", s.len(), s).into_bytes(),
            RespType::Array(None) => "*-1\r\n".to_string().into_bytes(),
            RespType::Array(Some(arr)) => {
                let mut result = format!("*{}\r\n", arr.len()).into_bytes();
                for item in arr {
                    result.extend(item.serialize());
                }
                result
            }
        }
    }
}

fn parse_simple_string(input: &mut BytesMut) -> Result<Option<RespType>, RespError> {
    if let Some(end) = find_crlf(input) {
        let line = str::from_utf8(&input[1..end])?.to_string();
        input.advance(end + 2);
        Ok(Some(RespType::SimpleString(line)))
    } else {
        Ok(None)
    }
}

fn parse_error(input: &mut BytesMut) -> Result<Option<RespType>, RespError> {
    if let Some(end) = find_crlf(input) {
        let line = str::from_utf8(&input[1..end])?.to_string();
        input.advance(end + 2);
        Ok(Some(RespType::Error(line)))
    } else {
        Ok(None)
    }
}

fn parse_integer(input: &mut BytesMut) -> Result<Option<RespType>, RespError> {
    if let Some(end) = find_crlf(input) {
        let num_str = str::from_utf8(&input[1..end])?;
        let num = num_str.parse::<i64>().map_err(|_| RespError::InvalidData)?;
        input.advance(end + 2);
        Ok(Some(RespType::Integer(num)))
    } else {
        Ok(None)
    }
}

fn parse_bulk_string(input: &mut BytesMut) -> Result<Option<RespType>, RespError> {
    if let Some(len_end) = find_crlf(input) {
        let len_str = str::from_utf8(&input[1..len_end])?;
        let len = len_str.parse::<i64>().map_err(|_| RespError::InvalidData)?;

        if len == -1 {
            input.advance(len_end + 2);
            return Ok(Some(RespType::BulkString(None)));
        }

        let len = len as usize;
        let total_len = len_end + 2 + len + 2;

        if input.len() < total_len {
            return Ok(None);
        }

        let string = str::from_utf8(&input[len_end + 2..len_end + 2 + len])?.to_string();
        input.advance(total_len);
        Ok(Some(RespType::BulkString(Some(string))))
    } else {
        Ok(None)
    }
}

fn parse_array(input: &mut BytesMut) -> Result<Option<RespType>, RespError> {
    if let Some(len_end) = find_crlf(input) {
        let len_str = str::from_utf8(&input[1..len_end])?;
        let len = len_str.parse::<i64>().map_err(|_| RespError::InvalidData)?;

        if len == -1 {
            input.advance(len_end + 2);
            return Ok(Some(RespType::Array(None)));
        }

        let len = len as usize;
        let mut pos = len_end + 2;
        let mut elements = Vec::with_capacity(len);

        for _ in 0..len {
            if pos >= input.len() {
                return Ok(None);
            }

            let mut rest = input.split_off(pos);
            std::mem::swap(input, &mut rest);

            match RespType::parse(input)? {
                Some(element) => {
                    pos = input.len();
                    elements.push(element);
                    let mut rest = rest;
                    rest.unsplit(input.clone());
                    *input = rest;
                }
                None => {
                    let mut rest = rest;
                    rest.unsplit(input.clone());
                    *input = rest;
                    return Ok(None);
                }
            }
        }

        Ok(Some(RespType::Array(Some(elements))))
    } else {
        Ok(None)
    }
}

fn find_crlf(input: &[u8]) -> Option<usize> {
    if input.len() < 2 {
        return None;
    }

    input.windows(2)
        .position(|window| window == b"\r\n")
} 