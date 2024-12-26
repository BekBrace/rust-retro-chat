// Import required libraries and modules
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};
use serde::{Serialize, Deserialize};
use chrono::Local;
use std::error::Error;

// Define message structure matching client's expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    username: String,
    content: String,
    timestamp: String,
    message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    UserMessage,
    SystemNotification,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize server on specified port
    let listener = TcpListener::bind("127.0.0.1:8082").await?;
    println!("╔════════════════════════════════════════╗");
    println!("║        RETRO CHAT SERVER ACTIVE        ║");
    println!("║        Port: 8082  Host: 127.0.0.1     ║");
    println!("║        Press Ctrl+C to shutdown        ║");
    println!("╚════════════════════════════════════════╝");

    // Create broadcast channel for message distribution
    // Channel size of 100 messages for backlog
    let (tx, _) = broadcast::channel::<String>(100);

    // Main server loop - accept and handle new connections
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("┌─[{}] New connection", Local::now().format("%H:%M:%S"));
        println!("└─ Address: {}", addr);

        // Clone sender for this connection
        let tx = tx.clone();
        let rx = tx.subscribe();

        // Spawn new task for each connection
        tokio::spawn(async move {
            handle_connection(socket, tx, rx).await
        });
    }
}

// Handle individual client connections
async fn handle_connection(
    mut socket: TcpStream,
    tx: broadcast::Sender<String>,
    mut rx: broadcast::Receiver<String>,
) {
    // Split socket into reader and writer
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut username = String::new();

    // Read username from client
    reader.read_line(&mut username).await.unwrap();
    let username = username.trim().to_string();

    // Create and send join notification
    let join_msg = ChatMessage {
        username: username.clone(),
        content: "joined the chat".to_string(),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        message_type: MessageType::SystemNotification,
    };
    let join_json = serde_json::to_string(&join_msg).unwrap();
    tx.send(join_json).unwrap();

    // Message handling loop
    let mut line = String::new();
    loop {
        tokio::select! {
            // Handle incoming messages from client
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break; // Client disconnected
                }
                // Create and broadcast user message
                let msg = ChatMessage {
                    username: username.clone(),
                    content: line.trim().to_string(),
                    timestamp: Local::now().format("%H:%M:%S").to_string(),
                    message_type: MessageType::UserMessage,
                };
                let json = serde_json::to_string(&msg).unwrap();
                tx.send(json).unwrap();
                line.clear();
            }
            // Handle broadcasting messages to all clients
            result = rx.recv() => {
                let msg = result.unwrap();
                writer.write_all(msg.as_bytes()).await.unwrap();
                writer.write_all(b"\n").await.unwrap();
            }
        }
    }

    // Create and send leave notification
    let leave_msg = ChatMessage {
        username: username.clone(),
        content: "left the chat".to_string(),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        message_type: MessageType::SystemNotification,
    };
    let leave_json = serde_json::to_string(&leave_msg).unwrap();
    tx.send(leave_json).unwrap();
    
    println!("└─[{}] {} disconnected", Local::now().format("%H:%M:%S"), username);
}
