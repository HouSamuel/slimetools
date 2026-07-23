mod slime_lib;

use std::env;
use slime_lib::is_slime_chunk;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("用法: cargo run -- <世界种子> <区块X> <区块Z>");
        std::process::exit(1);
    }

    let world_seed: i64 = args[1].parse().expect("世界种子必须是整数");
    let chunk_x: i32 = args[2].parse().expect("区块X必须是整数");
    let chunk_z: i32 = args[3].parse().expect("区块Z必须是整数");

    let result = is_slime_chunk(world_seed, chunk_x, chunk_z);
    println!("{}", if result { 1 } else { 0 });
}