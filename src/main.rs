use clap::Parser;

use purpy2::Args;

fn main() {
    env_logger::init();
    let args = Args::parse();

    if args.winit {
        pollster::block_on(purpy2::wgpu_main());
    } else {
        match purpy2::sdl_main(args) {
            Ok(_) => {}
            Err(e) => panic!("{}", e),
        }
    }
}
