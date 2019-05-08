use pcap::{Capture, Device};

type Error = pcap::Error;

pub fn run() -> Result<(), Error> {
    let main_device = Device {
        // name: String::from("enp4s0"),
        name: String::from("en8"),
        desc: None,
    };
    println!("{:#?}", main_device);
    let mut cap = Capture::from_device(main_device)?
        .promisc(true)
        .snaplen(0)
        .open()?;

    let mut total = 0;
    while let Ok(packet) = cap.next() {
        total += packet.len();
        println!("len: {:>10} B, total: {:>10} B", packet.len(), total);
    }

    Ok(())
}
