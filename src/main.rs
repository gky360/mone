use std::process;
use structopt::StructOpt;

fn main() {
    let result = {
        let opt = mone::Opt::from_args();
        mone::run(&opt)
    };

    process::exit(match result {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{:?}", err);
            1
        }
    })
}
