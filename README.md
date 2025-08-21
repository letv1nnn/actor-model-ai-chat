# Actor-based LLM chat

This project is a simple AI chat application with a TypeScript/HTML frontend and a Rust backend.
The frontend provides a chat UI where users can type messages, which are sent to the backend over HTTP.
The backend uses the Actor model to process each request, either forwarding it to an AI model (like Ollama) for a real response or falling back to an echo reply for development. I've created this chat to test my devops agent and 
study Actor Model.

## Build

**Clone**
```bash
git clone https://github.com/letv1nnn/actor-model-ai-chat.git
cd actor-model-ai-chat
```

**Build and Run**
```bash
cd ./backend/
cargo build --release
./target/release/chat.exe # for windows leave like that, for other OS, remove ".exe"
```

**Clean**
```bash
cargo clean 
```
