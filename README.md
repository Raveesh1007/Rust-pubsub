# Rust PubSub WebSocket Server

## Features
- Subscribe to topics over WebSocket
- Publish messages to topics (with optional key)
- Simple JSON protocol
- Easily extensible

## Getting Started

### Prerequisites
- [Rust](https://rustup.rs/) (stable)
- [websocat](https://github.com/vi/websocat) (for testing, optional)

### Setup
1. **Clone the repository**
   ```sh
   git clone <your-repo-url>
   cd rust pubsub/pubsub
   ```
2. **Create a `.env` file** in the project root (next to `Cargo.toml`):
   ```env
   PUBLISH_KEY=your_random_key_here
   PORT=3000
   ADDRESS=0.0.0.0:3000
   ```
   - Generate a random key in PowerShell:
     ```powershell
     -join ((1..24) | ForEach-Object { '{0:x2}' -f (Get-Random -Minimum 0 -Maximum 256) })
     ```

### Running the Server
```sh
cargo run
```
- The server will listen on the address/port specified in your `.env` (default: `0.0.0.0:3000`).

## Usage

### WebSocket Endpoint
- Connect to: `ws://localhost:3000/`

### Example Usage with websocat

#### 1. Subscribe to a topic
Open a terminal and run:
```sh
websocat ws://localhost:3000/
```
Then send:
```json
{ "subscribe": { "topics": ["test"] } }
```

#### 2. Publish a message
Open another terminal and run:
```sh
websocat ws://localhost:3000/
```
Then send (replace `YOUR_PUBLISH_KEY`):
```json
{ "publish": { "topics": ["test"], "message": "hello world", "key": "YOUR_PUBLISH_KEY" } }
```

#### 3. Expected Response
The subscriber terminal will receive:
```json
{ "message": { "topic": "test", "message": "hello world" } }
```

## Protocol

### Subscribe
```json
{ "subscribe": { "topics": ["sports", "weather"] } }
```

### Unsubscribe
```json
{ "unsubscribe": { "topics": ["sports"] } }
```

### Publish
```json
{ "publish": { "topics": ["weather"], "message": "storms ahead", "key": "YOUR_PUBLISH_KEY" } }
```

### Responses
- **Message:**
  ```json
  { "message": { "topic": "weather", "message": "storms ahead" } }
  ```
- **Error:**
  ```json
  { "error": { "message": "some error message here" } }
  ```

## License

Licensed under either of
- Apache License, Version 2.0
- MIT license
