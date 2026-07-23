/// 史莱姆区块测试程序
/// 
/// 使用方法:
/// cargo test --test slime_test -- --nocapture -- <命令> [参数...]
/// 
/// 命令列表:
/// check <seed> <chunkX> <chunkZ>      - 单个区块判定
/// generate <数量> <输出文件>           - 生成随机测试数据
/// batch <坐标文件>                     - 批量判定并输出01
/// compare <坐标文件>                   - 对比Rust和Java结果
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::Command;
use slimetools::slime_lib::is_slime_chunk;

/// 获取命令行参数（跳过测试框架的参数）
fn get_args() -> Vec<String> {
    let args: Vec<String> = env::args().collect();
    // 找到 "--" 分隔符，后面的是用户参数
    if let Some(idx) = args.iter().position(|s| s == "--") {
        args[idx + 1..].to_vec()
    } else {
        Vec::new()
    }
}

/// 单个区块判定
#[test]
fn check() {
    let args = get_args();
    if args.is_empty() || args[0] != "check" {
        return;
    }

    if args.len() != 4 {
        eprintln!("用法: cargo test --test slime_test -- --nocapture -- check <seed> <chunkX> <chunkZ>");
        return;
    }

    let seed: i64 = args[1].parse().expect("seed 必须是 i64 类型");
    let chunk_x: i32 = args[2].parse().expect("chunkX 必须是 i32 类型");
    let chunk_z: i32 = args[3].parse().expect("chunkZ 必须是 i32 类型");

    let result = is_slime_chunk(seed, chunk_x, chunk_z);
    println!("{}", result);
}

/// 生成随机测试数据
#[test]
fn generate() {
    let args = get_args();
    if args.is_empty() || args[0] != "generate" {
        return;
    }

    if args.len() != 3 {
        eprintln!("用法: cargo test --test slime_test -- --nocapture -- generate <数量> <输出文件>");
        return;
    }

    let count: usize = args[1].parse().expect("数量必须是正整数");
    let output_file = &args[2];

    let mut rng = rand::thread_rng();

    let mut file = File::create(output_file).expect("创建文件失败");

    for _ in 0..count {
        let seed = rand::Rng::gen_range(&mut rng, i64::MIN..=i64::MAX);
        let x = rand::Rng::gen_range(&mut rng, -1_000_000..=1_000_000);
        let z = rand::Rng::gen_range(&mut rng, -1_000_000..=1_000_000);
        writeln!(file, "{},{},{}", seed, x, z).expect("写入文件失败");
    }

    println!("已生成 {} 条测试数据到 {}", count, output_file);
}

/// 批量判定（Rust版）
#[test]
fn batch() {
    let args = get_args();
    if args.is_empty() || args[0] != "batch" {
        return;
    }

    if args.len() != 2 {
        eprintln!("用法: cargo test --test slime_test -- --nocapture -- batch <坐标文件>");
        return;
    }

    let filename = &args[1];
    let file = File::open(filename).expect("打开文件失败");
    let reader = BufReader::new(file);

    let mut result = String::new();

    for line in reader.lines() {
        let line = line.expect("读取行失败").trim().to_string();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            continue;
        }
        let seed: i64 = parts[0].parse().expect("解析seed失败");
        let x: i32 = parts[1].parse().expect("解析x失败");
        let z: i32 = parts[2].parse().expect("解析z失败");

        result.push(if is_slime_chunk(seed, x, z) { '1' } else { '0' });
    }

    println!("{}", result);
}

/// 对比测试
#[test]
fn compare() {
    let args = get_args();
    if args.is_empty() || args[0] != "compare" {
        return;
    }

    if args.len() != 2 {
        eprintln!("用法: cargo test --test slime_test -- --nocapture -- compare <坐标文件>");
        return;
    }

    let filename = &args[1];

    // 编译Java程序（在tests目录下编译）
    Command::new("javac")
        .current_dir("tests")
        .arg("check.java")
        .output()
        .expect("编译check.java失败");
    Command::new("javac")
        .current_dir("tests")
        .arg("batch.java")
        .output()
        .expect("编译batch.java失败");

    // Rust批量判定
    let file = File::open(filename).expect("打开文件失败");
    let reader = BufReader::new(file);

    let mut rust_result = String::new();

    for line in reader.lines() {
        let line = line.expect("读取行失败").trim().to_string();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            continue;
        }
        let seed: i64 = parts[0].parse().expect("解析seed失败");
        let x: i32 = parts[1].parse().expect("解析x失败");
        let z: i32 = parts[2].parse().expect("解析z失败");

        rust_result.push(if is_slime_chunk(seed, x, z) { '1' } else { '0' });
    }

    // Java批量判定（在tests目录下运行，调整相对路径）
    let java_filename = if filename.starts_with("tests/") {
        &filename[6..]
    } else {
        filename
    };
    let java_output = Command::new("java")
        .current_dir("tests")
        .arg("-cp")
        .arg(".")
        .arg("batch")
        .arg(java_filename)
        .output()
        .expect("调用Java程序失败");
    
    // 打印Java程序的错误输出
    if !java_output.status.success() {
        let stderr = String::from_utf8_lossy(&java_output.stderr);
        println!("Java错误: {}", stderr);
    }
    
    let java_result = String::from_utf8_lossy(&java_output.stdout).trim().to_string();

    println!("Rust: {}", rust_result);
    println!("Java: {}", java_result);
    println!("一致: {}", rust_result == java_result);

    assert_eq!(rust_result, java_result, "Rust和Java结果不一致");
}
