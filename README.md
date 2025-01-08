# Rust Redis Server Implementation

A lightweight Redis server implementation in Rust, supporting core Redis commands and features. This project implements the Redis Serialization Protocol (RESP) and provides basic key-value store functionality with support for expiring keys.

## Features

- RESP (Redis Serialization Protocol) implementation
- Concurrent client handling using Tokio async runtime
- In-memory key-value store with expiry support
- Support for both raw RESP protocol and redis-cli text format
- Thread-safe data storage using Arc and Mutex

### Supported Commands

- `PING` - Test server connection
- `ECHO <message>` - Echo back a message
- `SET <key> <value> [EX seconds | PX milliseconds]` - Set key with optional expiry
- `GET <key>` - Get value by key
- `EXISTS <key>` - Check if key exists
- `DEL <key> [key ...]` - Delete one or more keys
- `INCR <key>` - Increment numeric value
- `DECR <key>` - Decrement numeric value

## Prerequisites

- Rust 1.56 or later
- Cargo package manager
- Redis CLI (for testing, optional)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/apih99/redisServerRust.git
cd redisServerRust
```

2. Build the project:
```bash
cargo build --release
```

3. Run the server:
```bash
cargo run --release
```

The server will start listening on `127.0.0.1:6379` (default Redis port).

## Usage

### Using Redis CLI

1. Start the Redis server:
```bash
cargo run
```

2. In another terminal, use redis-cli to interact with the server:
```bash
redis-cli
```

3. Example commands:
```redis
PING
SET mykey "Hello World"
GET mykey
SET counter "10"
INCR counter
DECR counter
EXISTS mykey
DEL mykey
```

### Using Raw RESP Protocol

You can also interact with the server using raw RESP protocol:

```bash
echo "*1\r\n$4\r\nPING\r\n" | nc localhost 6379
echo "*3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$5\r\nvalue\r\n" | nc localhost 6379
```

## Project Structure

- `src/main.rs` - Server initialization and connection handling
- `src/resp/mod.rs` - RESP protocol implementation
- `src/command/mod.rs` - Command parsing and execution
- `src/store/mod.rs` - Key-value store implementation

### Core Components

1. **RESP Protocol (resp/mod.rs)**
   - Handles serialization and deserialization of Redis protocol
   - Supports all RESP data types: Simple Strings, Errors, Integers, Bulk Strings, and Arrays
   - Includes support for both RESP protocol and plain text commands

2. **Command Handler (command/mod.rs)**
   - Parses and executes Redis commands
   - Implements command-specific logic
   - Handles command validation and error responses

3. **Store (store/mod.rs)**
   - Thread-safe key-value store implementation
   - Handles key expiry
   - Supports atomic operations (INCR/DECR)

4. **Server (main.rs)**
   - Async TCP server implementation using Tokio
   - Handles client connections
   - Manages command processing pipeline

## Error Handling

The server implements robust error handling:
- Invalid commands return appropriate error messages
- Connection errors are properly handled and logged
- Expired keys are automatically removed
- Type mismatch errors for INCR/DECR operations

## Performance Considerations

- Uses Tokio for async I/O and concurrent client handling
- Implements buffered reading and writing for better performance
- Uses Arc and Mutex for thread-safe state management
- Efficient memory usage with BytesMut for buffer management

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by the original Redis implementation
- Built using the Tokio async runtime
- Uses various Rust crates including bytes, anyhow, and thiserror

## Future Improvements

- Implement persistence (SAVE command)
- Add support for Redis data structures (Lists, Sets, Hashes)
- Implement pub/sub functionality
- Add cluster support
- Implement more Redis commands
- Add comprehensive test suite
- Add metrics and monitoring
- Implement Redis modules support 