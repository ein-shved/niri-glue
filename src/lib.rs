//!
//! Basic utility types. The [Args] is core type which both handles command line
//! arguments and executes process. The main argument is
//! [command](Args::command). The command specify which glue type this utility
//! should provide. There could be optional [path](Args::path) argument which
//! specifies path to niri socket and [format](Args::format) argument which
//! specifies format of output messages (for each environment susbsystem could
//! be different formats).
//!
//! Each [command's](Command) emum type implements [Parser] and [Runner] traits
//! to parse arguments from one side and to perform action from another.
//!
#![warn(missing_docs)]

use clap::Subcommand;
pub use clap::{Parser, ValueEnum};
use niri_ipc::socket::Socket;
use std::path::PathBuf;

mod layout;

pub use layout::{Layout, SwitchLayout};

/// Top-level arguments structure
#[derive(Parser, Debug)]
#[command(
    author = "Yury Shvedov (github:ein-shved)",
    version = "0.1",
    about = "Glue for niri",
    long_about = "Utility to glue niri and environment like bars and pagers."
)]
pub struct Args {
    /// The procedure to run
    #[command(subcommand)]
    command: Command,

    /// Optional path to niri socket
    #[arg(short, long, help = "Path to niri socket")]
    path: Option<PathBuf>,

    /// The format of output messages
    #[arg(short, long, default_value = "waybar")]
    format: Format,
}

/// The list of supported commands
#[derive(Subcommand, Debug, Clone)]
#[command(about, long_about)]
pub enum Command {
    /// Check niri availability.
    ///
    /// Exits with success if niri is available and panics if niri is
    /// unavailable.
    #[command(about, long_about)]
    Test(TestSocket),

    /// Keyboard layout monitor.
    ///
    /// Produces to stdout messages about keyboard layout actions.
    #[command(about, long_about)]
    Layout(Layout),

    /// Keyboard layout switcher.
    ///
    /// Switches the keyboard layout.
    #[command(about, long_about)]
    SwitchLayout(SwitchLayout),
}

/// The list of available formats of output messages
#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum Format {
    /// The waybar custom module format (See https://github.com/Alexays/Waybar)
    Waybar,
}

/// The trait for subcommand
pub trait Runner {
    /// The [Args] will create socket for niri and pass it here
    fn run(self, socket: Socket, format: Format);
}

impl Args {
    /// Run chosen subcommand
    pub fn run(self) {
        let socket = if let Some(path) = self.path {
            Socket::connect_to(path).unwrap()
        } else {
            Socket::connect().unwrap()
        };
        match self.command {
            Command::Layout(cmd) => cmd.run(socket, self.format),
            Command::SwitchLayout(cmd) => cmd.run(socket, self.format),
            Command::Test(cmd) => cmd.run(socket, self.format),
        }
    }
}

/// Check niri availability.
#[derive(Parser, Debug, Clone)]
pub struct TestSocket {}

impl Runner for TestSocket {
    fn run(self, _socket: Socket, _format: Format) {}
}
