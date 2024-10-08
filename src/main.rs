pub mod mac;
pub mod sixaxis;

use clap::{Parser, Subcommand};
use mac::MACAddress;
use sixaxis::SixAxisController;

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

fn handle_get() {
    // connect to controller
    let controller = SixAxisController::open(None);
    if controller.is_err() {
        eprintln!("Failed to open controller: {}", controller.err().unwrap());
        std::process::exit(1);
    }
    let controller = controller.unwrap();

    // get paired mac
    let mac = controller.get_paired_mac();
    if mac.is_err() {
        eprintln!("Failed to get paired MAC: {}", mac.err().unwrap());
        std::process::exit(1);
    }
    let mac = mac.unwrap();

    println!("Paired MAC: {}", mac);
    std::process::exit(0);
}

fn handle_pair(mac: String) {
    // parse mac address
    let mac = MACAddress::from_string(&mac);
    if mac.is_err() {
        eprintln!("Invalid MAC Address: {}", mac.err().unwrap());
        std::process::exit(1);
    }
    let mac = mac.unwrap();

    // connect to controller
    let controller = SixAxisController::open(None);
    if controller.is_err() {
        eprintln!("Failed to open controller: {}", controller.err().unwrap());
        std::process::exit(1);
    }
    let controller = controller.unwrap();

    // pair controller
    let result = controller.set_paired_mac(&mac);
    if result.is_err() {
        eprintln!("Failed to pair controller: {}", result.err().unwrap());
        std::process::exit(1);
    }

    println!("Controller paired to MAC: {}", mac);
    std::process::exit(0);
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Get {} => handle_get(),
        Command::Pair { mac } => handle_pair(mac),
    }
}
