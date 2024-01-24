use clap::Parser;
use purpy2::winit_main;
use purpy2::Args;

fn main() {
    env_logger::init();
    let args = Args::parse();

    match pollster::block_on(winit_main(args)) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
