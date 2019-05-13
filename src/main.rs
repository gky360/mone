use mone::Opt;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();

    match mone::run(&opt) {
        Ok(()) => (),
        Err(err) => println!("{:?}", err),
    }
}
