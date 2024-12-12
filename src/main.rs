use niri_glue::{Args, Parser};

fn main() {
    let args = Args::parse();

    args.run();
}
