pub mod mac;
pub mod sixaxis;

use clap::{Parser, Subcommand};
use mac::MACAddress;
use sixaxis::{SixAxisController, USBDeviceId};

#[derive(Parser, Debug)]
#[command(version)]
#[command(
    about = "A simple CLI tool to view and change the paired MAC address of a Sony Sixaxis controller."
)]
struct Args {
    /// The subcommand to run.
    #[command(subcommand)]
    command: Command,

    /// Manually specify the USB device ID of the controller.
    #[arg(short, long)]
    vendor_id: Option<u16>,

    /// Manually specify the USB product ID of the controller.
    #[arg(short, long)]
    product_id: Option<u16>,
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

fn handle_get(device_id: Option<USBDeviceId>) {
    // connect to controller
    let controller = SixAxisController::open(device_id);
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

fn handle_pair(device_id: Option<USBDeviceId>, mac: String) {
    // parse mac address
    let mac = MACAddress::from_string(&mac);
    if mac.is_err() {
        eprintln!("Invalid MAC Address: {}", mac.err().unwrap());
        std::process::exit(1);
    }
    let mac = mac.unwrap();

    // connect to controller
    let controller = SixAxisController::open(device_id);
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

    // unwrap manually specified device id
    // if either vendor or product id is specified, both must be specified
    if args.vendor_id.is_some() != args.product_id.is_some() {
        eprintln!("Both vendor and product ID must be specified.");
        std::process::exit(1);
    }

    let device_id = if args.vendor_id.is_some() {
        Some(USBDeviceId {
            vendor: args.vendor_id.unwrap(),
            product: args.product_id.unwrap(),
        })
    } else {
        None
    };

    // handle subcommand
    match args.command {
        Command::Get {} => handle_get(device_id),
        Command::Pair { mac } => handle_pair(device_id, mac),
    }
}
