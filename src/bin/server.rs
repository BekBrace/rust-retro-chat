// Import required libraries and modules
// The tokio library provides asynchronous runtime for managing tasks and I/O operations efficiently
use tokio::{
    net::{TcpListener, TcpStream}, // Asynchronous TCP networking
    sync::broadcast,              // Broadcast channel for communication between tasks
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, // Async I/O utilities
};
use serde::{Serialize, Deserialize}; // Serialization and deserialization for structured data
use chrono::Local; // For working with local date and time
use std::error::Error; // Error handling trait

// Define the structure of chat messages
// This matches the data structure expected by clients and simplifies message handling
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    username: String,         // Name of the user sending the message
    content: String,          // Content of the message
    timestamp: String,        // Timestamp of when the message was sent
    message_type: MessageType, // Type of message (user or system notification)
}

// Define an enumeration for message types
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    UserMessage,              // Represents a message from a user
    SystemNotification,       // Represents system-generated messages (e.g., join/leave notifications)
}

// Main function using Tokio's asynchronous runtime
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Bind the server to the specified IP and port
    let listener = TcpListener::bind("127.0.0.1:8082").await?;
    
    // Display server startup message with formatting
    println!("╔════════════════════════════════════════╗");
    println!("║        RETRO CHAT SERVER ACTIVE        ║");
    println!("║        Port: 8082  Host: 127.0.0.1     ║");
    println!("║        Press Ctrl+C to shutdown        ║");
    println!("╚════════════════════════════════════════╝");

    // Create a broadcast channel for message distribution
    // This allows multiple subscribers to receive the same messages
    let (tx, _) = broadcast::channel::<String>(100); // Buffer size of 100 messages

    // Main server loop to handle incoming connections
    loop {
        let (socket, addr) = listener.accept().await?; // Accept a new connection
        
        // Display connection information
        println!("┌─[{}] New connection", Local::now().format("%H:%M:%S"));
        println!("└─ Address: {}", addr);

        // Clone sender for this connection and subscribe a receiver
        let tx = tx.clone();
        let rx = tx.subscribe();

        // Spawn a new task to handle this connection asynchronously
        tokio::spawn(async move {
            handle_connection(socket, tx, rx).await
        });
    }
}

// Function to handle individual client connections
async fn handle_connection(
    mut socket: TcpStream,               // The TCP connection for the client
    tx: broadcast::Sender<String>,      // Sender for broadcasting messages
    mut rx: broadcast::Receiver<String>, // Receiver for incoming broadcasts
) {
    // Split the socket into reader and writer parts
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader); // Buffer the reader for efficient I/O
    let mut username = String::new(); // Store the username sent by the client

    // Read the username sent by the client
    reader.read_line(&mut username).await.unwrap();
    let username = username.trim().to_string(); // Remove extra spaces or newlines

    // Send a system notification indicating the user has joined
    let join_msg = ChatMessage {
        username: username.clone(),
        content: "joined the chat".to_string(),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        message_type: MessageType::SystemNotification,
    };
    let join_json = serde_json::to_string(&join_msg).unwrap();
    tx.send(join_json).unwrap();

    // Initialize a buffer for incoming messages from the client
    let mut line = String::new();
    loop {
        tokio::select! {
            // Handle messages sent by the client
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break; // Exit loop if the client disconnects
                }
                // Create and broadcast a user message
                let msg = ChatMessage {
                    username: username.clone(),
                    content: line.trim().to_string(),
                    timestamp: Local::now().format("%H:%M:%S").to_string(),
                    message_type: MessageType::UserMessage,
                };
                let json = serde_json::to_string(&msg).unwrap();
                tx.send(json).unwrap();
                line.clear(); // Clear the buffer for the next message
            }
            // Handle incoming broadcasts and send them to the client
            result = rx.recv() => {
                let msg = result.unwrap();
                writer.write_all(msg.as_bytes()).await.unwrap();
                writer.write_all(b"\n").await.unwrap();
            }
        }
    }

    // Send a system notification indicating the user has left
    let leave_msg = ChatMessage {
        username: username.clone(),
        content: "left the chat".to_string(),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        message_type: MessageType::SystemNotification,
    };
    let leave_json = serde_json::to_string(&leave_msg).unwrap();
    tx.send(leave_json).unwrap();
    
    // Log disconnection information
    println!("└─[{}] {} disconnected", Local::now().format("%H:%M:%S"), username);
}
