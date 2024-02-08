use clap::Parser;
use purpy::winit_main;
use purpy::Args;

fn main() {
    env_logger::init();
    let args = Args::parse();

    match pollster::block_on(winit_main(args)) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
