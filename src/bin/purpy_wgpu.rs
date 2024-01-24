use clap::Parser;
use purpy2::Args;

fn main() {
    env_logger::init();
    let args = Args::parse();

    match purpy2::wgpu_main(args) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
