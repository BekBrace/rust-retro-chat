// Importing various modules from the cursive library for UI development
use cursive::{
    align::HAlign, // Horizontal alignment utilities
    event::Key, // Handling key press events
    theme::{BorderStyle, Palette, PaletteColor, Theme, Color, BaseColor}, // Styling components
    traits::*, // Additional traits for UI components
    views::{Dialog, EditView, LinearLayout, ScrollView, TextView, Panel, DummyView}, // UI elements
    Cursive, // Main Cursive application object
};

// Importing Serde for serialization and deserialization
use serde::{Deserialize, Serialize};

// Importing necessary standard library modules
use std::{env, error::Error, sync::Arc};

// Importing Tokio async utilities
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, // Asynchronous I/O utilities
    net::TcpStream, // For TCP connections
    sync::Mutex, // Provides thread-safe mutable access
};

// Importing Chrono for date and time handling
use chrono::Local;

// Struct to represent a chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    username: String, // Username of the sender
    content: String, // Message content
    timestamp: String, // Timestamp of when the message was sent
    message_type: MessageType, // Type of the message (e.g., user message or system notification)
}

// Enum to represent different types of messages
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    UserMessage, // Regular message from a user
    SystemNotification, // System-generated notification
}

// Main asynchronous entry point of the application
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Fetching username from command-line arguments
    let username = env::args()
        .nth(1) // Gets the second argument (after the program name)
        .expect("Please provide a username as argument"); // Exits if no username is provided

    // Initializing the Cursive UI framework
    let mut siv = cursive::default();
    siv.set_theme(create_retro_theme()); // Applying a custom retro theme

    // Creating a header to display chat title and username
    let header = TextView::new(format!(r#"╔═ RETRO CHAT ═╗ User: {} ╔═ {} ═╗"#,
        username, // Insert username
        Local::now().format("%H:%M:%S") // Insert current time
    ))
    .style(Color::Light(BaseColor::Green)) // Green text for retro look
    .h_align(HAlign::Center); // Center-align the header

    // Creating a message area with a scrollable text view
    let messages = TextView::new("") // Initialize empty text view
        .with_name("messages") // Assign a name for later access
        .min_height(20) // Minimum height for the message area
        .scrollable(); // Enable scrolling

    let messages = ScrollView::new(messages)
        .scroll_strategy(cursive::view::ScrollStrategy::StickToBottom) // Keep the scroll at the bottom
        .min_width(60) // Minimum width
        .full_width(); // Occupy full width of the parent

    // Creating an input area for typing messages
    let input = EditView::new()
        .on_submit(move |s, text| send_message(s, text.to_string())) // Define submit behavior
        .with_name("input") // Assign a name for later access
        .min_width(50) // Minimum width
        .max_height(3) // Limit input height to 3 lines
        .full_width(); // Occupy full width of the parent

    // Creating help text for user commands
    let help_text = TextView::new("ESC:quit | Enter:send | Commands: /help, /clear, /quit")
        .style(Color::Dark(BaseColor::White)); // Styled with white text

    // Assembling the main layout
    let layout = LinearLayout::vertical()
        .child(Panel::new(header)) // Header panel
        .child(
            Dialog::around(messages) // Dialog box for messages
                .title("Messages") // Add title
                .title_position(HAlign::Center) // Center-align title
                .full_width()
        )
        .child(
            Dialog::around(input) // Dialog box for input
                .title("Message") // Add title
                .title_position(HAlign::Center) // Center-align title
                .full_width()
        )
        .child(Panel::new(help_text).full_width()); // Panel for help text

    // Wrapping layout for centering
    let centered_layout = LinearLayout::horizontal()
        .child(DummyView.full_width()) // Dummy views for spacing
        .child(layout)
        .child(DummyView.full_width());

    // Adding the centered layout to the Cursive root
    siv.add_fullscreen_layer(centered_layout);

    // Adding global key bindings
    siv.add_global_callback(Key::Esc, |s| s.quit()); // Quit on ESC
    siv.add_global_callback('/', |s| {
        s.call_on_name("input", |view: &mut EditView| {
            view.set_content("/"); // Insert '/' in input box
        });
    });

    // Establishing a connection to the chat server
    let stream = TcpStream::connect("127.0.0.1:8082").await?;
    let (reader, mut writer) = stream.into_split(); // Split stream into reader and writer

    writer.write_all(format!("{}\n", username).as_bytes()).await?; // Send username to server

    let writer = Arc::new(Mutex::new(writer)); // Thread-safe writer
    let writer_clone = Arc::clone(&writer); // Clone writer for later use
    siv.set_user_data(writer); // Store writer in the Cursive app data

    let reader = BufReader::new(reader); // Wrap reader with a buffer
    let mut lines = reader.lines(); // Line reader
    let sink = siv.cb_sink().clone(); // Clone Cursive callback sink

    // Spawn an async task to handle incoming messages
    tokio::spawn(async move {
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(msg) = serde_json::from_str::<ChatMessage>(&line) {
                // Format incoming message based on type
                let formatted_msg = match msg.message_type {
                    MessageType::UserMessage => format!("┌─[{}]\n└─ {} ▶ {}\n",
                        msg.timestamp, msg.username, msg.content),
                    MessageType::SystemNotification => format!("\n[{} {}]\n",
                        msg.username, msg.content),
                };
                // Update UI with the new message
                if sink.send(Box::new(move |siv: &mut Cursive| {
                    siv.call_on_name("messages", |view: &mut TextView| {
                        view.append(formatted_msg); // Append the message
                    });
                })).is_err() {
                    break; // Exit loop on error
                }
            }
        }
    });

    siv.run(); // Run the Cursive event loop
    let _ = writer_clone.lock().await.shutdown().await; // Close the writer
    Ok(()) // Exit successfully
}

// Function to handle sending messages
fn send_message(siv: &mut Cursive, msg: String) {
    if msg.is_empty() { // Ignore empty messages
        return;
    }

    // Handle specific commands
    match msg.as_str() {
        "/help" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.append("\n=== Commands ===\n/help - Show this help\n/clear - Clear messages\n/quit - Exit chat\n\n");
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content("");
            });
            return;
        }
        "/clear" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.set_content(""); // Clear messages
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content(""); // Clear input
            });
            return;
        }
        "/quit" => {
            siv.quit(); // Quit the application
            return;
        }
        _ => {}
    }

    // Send the message to the server
    let writer = siv.user_data::<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>().unwrap().clone();
    tokio::spawn(async move {
        let _ = writer.lock().await.write_all(format!("{}\n", msg).as_bytes()).await;
    });

    // Clear the input field
    siv.call_on_name("input", |view: &mut EditView| {
        view.set_content("");
    });
}

// Function to create a retro-style theme
fn create_retro_theme() -> Theme {
    let mut theme = Theme::default();
    theme.shadow = true; // Enable shadows
    theme.borders = BorderStyle::Simple; // Use simple borders
    
    let mut palette = Palette::default();
    palette[PaletteColor::Background] = Color::Rgb(0, 0, 20); // Deep blue background
    palette[PaletteColor::View] = Color::Rgb(0, 0, 20); // Deep blue for views
    palette[PaletteColor::Primary] = Color::Rgb(0, 255, 0); // Bright green text
    palette[PaletteColor::TitlePrimary] = Color::Rgb(0, 255, 128); // Green for titles
    palette[PaletteColor::Secondary] = Color::Rgb(255, 191, 0); // Amber secondary elements
    palette[PaletteColor::Highlight] = Color::Rgb(0, 255, 255); // Cyan highlights
    palette[PaletteColor::HighlightInactive] = Color::Rgb(0, 128, 128); // Dark cyan for inactive
    palette[PaletteColor::Shadow] = Color::Rgb(0, 0, 40); // Subtle shadow
    theme.palette = palette; // Apply the palette
    theme
}
