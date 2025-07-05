UpQuack: A Terminal URL Monitoring Application


![upquack_menu](https://github.com/user-attachments/assets/6e34cbb1-b305-4630-9d60-7f8290cf05e5)


UpQuack is a minimalist, real-time URL monitoring application built with Rust. It provides a clean and interactive Terminal User Interface (TUI) to keep track of your essential websites' uptime and response times. Whether you're a developer, a sysadmin, or just someone who wants to ensure their favorite sites are up, UpQuack offers a simple yet powerful solution.
✨ Features

    Real-time Monitoring: Continuously sends HEAD requests to specified URLs at defined intervals.

    Uptime Status: Displays UP, DOWN, UNKNOWN, or Error status for each monitored domain.

    HTTP Code & Response Time: Shows the last HTTP status code and response time (in milliseconds) for successful checks.

    Detailed History: View a chronological log of check statuses for each domain, including timestamps, status, HTTP code, response time, and error messages.

    Add/Delete Domains: Easily manage your list of monitored URLs directly from the TUI.

    Persistence: All monitored domains and their check histories are automatically saved to a local JSON file (db/domains.json) and loaded on startup.

    Responsive TUI: Built with ratatui and crossterm for a smooth and interactive terminal experience.

    Asynchronous Operations: Leverages tokio for efficient, non-blocking network requests, ensuring the UI remains responsive while monitoring runs in the background.

🚀 Getting Started
Prerequisites

Before you begin, ensure you have the following installed:

    Rust: If you don't have Rust installed, you can get it via rustup:

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

    (Follow the on-screen instructions to complete the installation.)

    Cargo: Rust's package manager, installed automatically with Rust.

Installation and Running

    Clone the repository:

    git clone <repository_url> # Replace <repository_url> with the actual URL
    cd upquack

    Build and run the application:

    cargo run

    This command will compile the project and then run the TUI application.

🕹️ Usage

Once the application starts, you will be presented with the main menu.
Main Menu

    E: Enter the "Monitored URLs" screen to manage and view your domains.

    Q: Quit the application.

Monitored URLs Screen

This screen displays a table of all your monitored domains, their current status, and last check details.

    A: Add a new domain.

    D: Delete the currently selected domain.

    H: View the detailed history of the currently selected domain.

    Up / j: Move selection up.

    Down / k: Move selection down.

    Esc: Return to the Main Menu.

Add New Domain

When you press A on the Monitored URLs screen, a popup will appear for entering a new URL.

    Type: Enter the URL (e.g., https://example.com).

    Enter: Confirm and add the domain.

    Esc: Cancel and close the popup without adding a domain.

Domain History

When you press H on a selected domain, this screen shows a detailed log of its past checks.

    Up / j: Scroll up through the history.

    Down / k: Scroll down through the history.

    Esc: Return to the Monitored URLs screen.

⚙️ Configuration and Persistence

UpQuack automatically saves and loads your monitored domains and their check histories.

    Data File: All data is stored in db/domains.json. This file is created automatically if it doesn't exist.

    History Limit: Each domain's check history is capped at the last 100 entries to prevent the file from growing indefinitely.

🛠️ Project Structure (Key Modules)

    src/main.rs: Application entry point, sets up the Tokio runtime and the TUI.

    src/app.rs: Defines the main App structure, handles global key events, and manages screen transitions.

    src/monitor.rs: Contains the core logic for asynchronously monitoring URLs and updating domain statuses.

    src/ui/domains.rs: Defines the DomainScreen struct, which manages the list of monitored domains, their state, and handles domain-specific UI interactions and persistence. It also defines the MonitoredDomain and CheckStatus data structures.

    src/ui/domain_table.rs: Renders the table of domains.

    src/ui/history_screen.rs: Renders the detailed history for a selected domain.

    src/ui/popup.rs: Generic popup component for input.

    src/utils.rs: Utility functions, e.g., URL validation.

📦 Dependencies

This project relies on the following key crates:

    tokio: Asynchronous runtime for concurrent operations.

    ratatui: A Rust library for building TUIs.

    crossterm: A pure-Rust, cross-platform terminal manipulation library.

    reqwest: An ergonomic, batteries-included HTTP client.

    chrono: Date and time library for timestamps.

    serde: Serialization/deserialization framework for saving/loading data.

    serde_json: JSON support for serde.

    uuid: For generating unique IDs for domains.

    tui-textarea: For text input fields in the TUI.

    log: For logging messages (errors, info).

🤝 Contributing

Contributions are welcome! If you have suggestions for improvements, new features, or bug fixes, please feel free to:

    Fork the repository.

    Create a new branch (git checkout -b feature/your-feature-name).

    Make your changes.

    Commit your changes (git commit -m 'Add new feature').

    Push to the branch (git push origin feature/your-feature-name).

    Open a Pull Request.

📄 License

This project is licensed under the MIT License - see the LICENSE file for details
