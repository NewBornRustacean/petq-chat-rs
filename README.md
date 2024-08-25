# petq-chat-rs
A simple openai chat streaming server written in rust
## Features
- [x] async chat streaming
- [ ] communicate with mongodb
- [ ] more
## How to start
1. Install rust on your machine 
    - [official](https://www.rust-lang.org/tools/install)
    - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Set a OpenAI_API_KEY
    ```
    # On macOS/Linux
    export OPENAI_API_KEY='sk-...'
    ```
3. cargo run
    ```
    cd petq-chat-rs
    cargo run --release
    ```
4. curl example
    - `GET http://localhost:3000/chat-stream/{userid(uuid}/{chatid(uiuid}?prompt='{user prompt}'`
    - `GET http://localhost:3000/chat-stream/9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d/9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d?prompt='My dog is vomitting. Should I go to a vet?'`
5. Swagger-ui
    - `http://localhost:3000/swagger-ui`
