use clap::Parser;
use purpy::Args;

fn main() {
    env_logger::init();
    let args = Args::parse();

    match purpy::sdl_main(args) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
