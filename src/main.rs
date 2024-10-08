pub mod ds4;
pub mod mac;

use clap::{Parser, Subcommand};
use ds4::{get_mac, set_mac};
use mac::MACAddress;

#[derive(Parser, Debug)]
#[command(version)]
#[command(
    about = "A simple CLI tool to view and change the paired MAC address of a Sony Sixaxis controller."
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Get and print the current paired MAC address.
    Get {},

    /// Pair the controller to a new MAC address.
    Pair {
        /// The MAC address to pair the controller to.
        mac: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Get {} => match get_mac() {
            Ok(mac) => println!("Current MAC address: {}", mac),
            Err(e) => eprintln!("Error: {}", e),
        },
        Command::Pair { mac } => match MACAddress::from_string(&mac) {
            Ok(mac) => match set_mac(&mac) {
                Ok(_) => println!("Successfully paired controller to MAC address: {}", mac),
                Err(e) => eprintln!("Error: {}", e),
            },
            Err(e) => eprintln!("Error: {}", e),
        },
    }
}
