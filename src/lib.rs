use pnet::datalink;
use pnet::datalink::{Channel::Ethernet, NetworkInterface};
use std::error;

type Error = Box<error::Error>;

pub fn run() -> Result<(), Error> {
    let interface_name = "en8";
    let interface_names_match = |iface: &NetworkInterface| iface.name == interface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap();

    // Create a new channel, dealing with layer 2 packets
    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };

    let mut total = 0;
    loop {
        match rx.next() {
            Ok(packet) => {
                let len = packet.len();
                total += len;
                println!("len: {:>10} B, total: {:>10} B", len, total);
            }
            Err(err) => return Err(Box::new(err)),
        }
    }
}
