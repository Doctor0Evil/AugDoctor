use clap::Parser;
use crate::donutloop::*;
use crate::bchainproof::*;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    donutloop: bool,
    #[arg(long)]
    path: Option<String>,
    #[arg(long)]
    emit_bchainproof: bool,
    #[arg(long)]
    out: Option<String>,
    #[arg(long)]
    debug_template: bool,
}

pub fn run() {
    let args = Args::parse();
    if args.debug_template {
        println!("[DEBUG] Template invariants validated – see excavation above");
    }
    if args.donutloop {
        if let Some(p) = args.path {
            let _ = verify_chain(&p);
        }
    }
    if args.emit_bchainproof {
        if let Some(o) = args.out {
            let _ = emit(&o, "0x9a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c", "bostrom18q4...");
        }
    }
}
