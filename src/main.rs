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

    /// Do not print device information.
    #[arg(short, long, default_value = "false")]
    no_device_info: bool,

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

        /// Skip verification of the paired MAC address.
        #[arg(short, long, default_value = "false")]
        no_verify: bool,
    },
}

fn connect_controller(
    device_id: Option<USBDeviceId>,
    print_device_info: bool,
) -> SixAxisController {
    let controller = SixAxisController::open(device_id);
    if controller.is_err() {
        eprintln!("Failed to open controller: {}", controller.err().unwrap());
        std::process::exit(1);
    }
    let controller = controller.unwrap();

    if print_device_info {
        let display_name = controller.get_display_name(Some(true));
        println!("Connected to: {}", display_name);
    }

    return controller;
}

fn handle_get(device_id: Option<USBDeviceId>, no_device_info: bool) {
    let controller = connect_controller(device_id, !no_device_info);

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

fn handle_pair(device_id: Option<USBDeviceId>, no_device_info: bool, verify: bool, mac: String) {
    // parse mac address
    // do this before connecting to controller to fail early
    let mac = MACAddress::from_string(&mac);
    if mac.is_err() {
        eprintln!("Invalid MAC Address: {}", mac.err().unwrap());
        std::process::exit(1);
    }
    let mac = mac.unwrap();

    // connect to controller
    let controller = connect_controller(device_id, !no_device_info);

    // pair controller
    let result = controller.set_paired_mac(&mac);
    if result.is_err() {
        eprintln!("Failed to pair controller: {}", result.err().unwrap());
        std::process::exit(1);
    }

    if verify {
        // fetch paired mac again to verify
        let paired_mac = controller.get_paired_mac();
        if paired_mac.is_err() {
            eprintln!("Failed to get paired MAC: {}", paired_mac.err().unwrap());
            std::process::exit(1);
        }
        let paired_mac = paired_mac.unwrap();

        if paired_mac != mac {
            eprintln!(
                "Failed to verify paired MAC: expected {}, got {}",
                mac, paired_mac
            );
            std::process::exit(1);
        }
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
        Command::Get {} => handle_get(device_id, args.no_device_info),
        Command::Pair { mac, no_verify } => {
            handle_pair(device_id, args.no_device_info, !no_verify, mac)
        }
    }
}
