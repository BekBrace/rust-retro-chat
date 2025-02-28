# 📟 Rust Retro Chat

A retro-styled terminal chat application built with Rust, showcasing async programming and the Cursive TUI library.

## Features
 
- 🎨 Retro terminal UI using Cursive
- 👥 Multiple concurrent users
- 🚀 Async networking with Tokio
- 📢 Join/Leave notifications
- 🌈 Colored messages and UI elements
- ⌚ Timestamp for messages

## Requirements

- Rust (latest stable version)
- Linux/Unix system with ncurses development libraries

## Installation

1. Install ncurses development libraries (required for Cursive):
```bash
# Ubuntu/Debian
sudo apt-get install libncursesw5-dev

# Fedora
sudo dnf install ncurses-devel
```

2. Build the project:
```bash
cargo build --release
```

## Running the Application

1. Start the server:
```bash
cargo run --bin server
```

2. In different terminals, start clients:
```bash
cargo run --bin client <username>
```

Replace `<username>` with your desired username.

## Controls

- Type your message and press Enter to send
- Press 'q' or Esc to quit
- Messages window automatically scrolls to show new messages
- System notifications for users joining/leaving are highlighted

## Implementation Details

- Uses Tokio for async networking
- Cursive for the retro terminal user interface
- JSON message format for communication
- Broadcast channel for message distribution
- Thread-safe message handling
