// ==============================================
//                全局硬编码配置区
// ==============================================
const WORLD_SEED: i128 = 20260627;
const CENTER_X: i32 = 0;
const CENTER_Z: i32 = 0;
const RADIUS: i32 = 100;

/// 匹配模式矩阵（二维，行优先）
/// 0 = 任意（不检查）
/// 1 = 必须为史莱姆区块
/// 2 = 必须为非史莱姆区块
const PATTERN: [[u8; 3]; 3] = [
    [2, 1, 2],
    [2, 1, 2],
    [1, 2, 1],
];
const PATTERN_WIDTH: usize = 3;
const PATTERN_HEIGHT: usize = 3;

/// 匹配目标：0 = 找出所有匹配，1 = 找到一个即停止，2 = 找到两个即停止
const MATCH_TARGET: usize = 0;

/// 是否输出完整种子文件（0/1 矩阵）：true = 输出，false = 不输出
const OUTPUT_SEED_FILE: bool = true;

// MC史莱姆区块公式固定系数
const C_X2: i64 = 4987142;
const C_X1: i64 = 5947611;
const C_Z2: i64 = 4392871;
const C_Z1: i64 = 389711;
const XOR_MASK: i64 = 987234911;
const BOUND: i32 = 10;
const TARGET: i32 = 0;

// Java Random LCG常数
const LCG_MULT: u64 = 0x5DEECE66D;
const LCG_ADD: u64 = 0xB;
const LCG_MASK: u64 = (1 << 48) - 1;
const XOR_SEED: u64 = 0x5DEECE66D;

use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;
use std::path::Path;

// ============================================================
//  FastRandom：模拟 Java Random（高度优化内联）
// ============================================================
#[derive(Clone, Copy)]
#[repr(transparent)]
struct FastRandom {
    seed: u64,
}

impl FastRandom {
    #[inline(always)]
    fn set_seed(&mut self, seed: i64) {
        self.seed = ((seed as u64) ^ XOR_SEED) & LCG_MASK;
    }

    #[inline(always)]
    fn next31(&mut self) -> u32 {
        self.seed = self.seed
            .wrapping_mul(LCG_MULT)
            .wrapping_add(LCG_ADD)
            & LCG_MASK;
        (self.seed >> 17) as u32
    }

    #[inline(always)]
    fn next_int_mod(&mut self, bound: i32) -> i32 {
        let bits = self.next31() as i64;
        (bits % bound as i64) as i32
    }
}

// ============================================================
//  辅助函数
// ============================================================

/// 计算整数占用的字符宽度（用于对齐）
#[inline(always)]
fn num_width(mut n: i32) -> usize {
    if n == 0 {
        return 1;
    }
    let mut w = if n < 0 { 1 } else { 0 };
    n = n.abs();
    while n > 0 {
        w += 1;
        n /= 10;
    }
    w
}

/// 构建种子文件头部
fn build_seed_header(
    min_x: i32,
    max_x: i32,
    min_z: i32,
    max_z: i32,
    x_count: usize,
    z_count: usize,
    cell_width: usize,
    z_col_width: usize,
    x_headers: &[Vec<u8>],
    seed: i64,
) -> Vec<u8> {
    let mut h = Vec::with_capacity(512);
    write!(
        &mut h,
        "Seed: {}\n\
         X range: [{}, {}]  Z range: [{}, {}]\n\
         Dimensions: {} × {}\n\
         Rule: nextInt({}) == {}\n\
         {:<w$}",
        seed, min_x, max_x, min_z, max_z, x_count, z_count, BOUND, TARGET,
        "", w = z_col_width
    )
    .unwrap();
    for hdr in x_headers {
        h.extend_from_slice(hdr);
    }
    h.push(b'\n');
    h.extend(std::iter::repeat(b' ').take(z_col_width));
    h.extend(std::iter::repeat(b'-').take(x_count * cell_width));
    h.push(b'\n');
    h
}

/// 输出完整种子文件（全缓冲并行写入，每行独立计算）
fn output_seed_file(
    min_x: i32,
    max_x: i32,
    min_z: i32,
    max_z: i32,
    x_count: usize,
    z_count: usize,
    cell_width: usize,
    z_col_width: usize,
    x_headers: &[Vec<u8>],
    z_headers: &[Vec<u8>],
    fx_arr: &[i64],
    fz_arr: &[i64],
    zero_cell: &[u8],
    one_cell: &[u8],
    filename: &Path,
    seed: i64,
) -> std::io::Result<()> {
    let num_cpus = rayon::current_num_threads();
    let chunk_size = (z_count + num_cpus - 1) / num_cpus;

    // 并行生成每个块的数据
    let block_results: Vec<Vec<u8>> = (0..num_cpus)
        .into_par_iter()
        .map(|block_id| {
            let start_z_idx = block_id * chunk_size;
            let end_z_idx = std::cmp::min(start_z_idx + chunk_size, z_count);
            if start_z_idx >= z_count {
                return Vec::new();
            }
            let rows = end_z_idx - start_z_idx;
            let row_len = z_col_width + x_count * cell_width + 1;
            let mut block_buf = Vec::with_capacity(rows * row_len);
            let mut rng = FastRandom { seed: 0 };
            let mut line_buf = Vec::with_capacity(row_len);

            for z_idx in start_z_idx..end_z_idx {
                line_buf.clear();
                unsafe {
                    line_buf.extend_from_slice(z_headers.get_unchecked(z_idx));
                    let fz = *fz_arr.get_unchecked(z_idx);
                    for x_idx in 0..x_count {
                        let fx = *fx_arr.get_unchecked(x_idx);
                        let combined = (seed as u64)
                            .wrapping_add(fx as u64)
                            .wrapping_add(fz as u64) as i64;
                        rng.set_seed(combined ^ XOR_MASK);
                        if rng.next_int_mod(BOUND) == TARGET {
                            line_buf.extend_from_slice(one_cell);
                        } else {
                            line_buf.extend_from_slice(zero_cell);
                        }
                    }
                }
                line_buf.push(b'\n');
                block_buf.extend_from_slice(&line_buf);
            }
            block_buf
        })
        .collect();

    // 写入头部 + 所有块
    let header = build_seed_header(
        min_x, max_x, min_z, max_z, x_count, z_count, cell_width, z_col_width, x_headers, seed,
    );
    let mut output = header;
    for block in block_results {
        output.extend_from_slice(&block);
    }

    let file = File::create(filename)?;
    let mut writer = BufWriter::with_capacity(1024 * 1024, file);
    writer.write_all(&output)?;
    writer.flush()?;
    Ok(())
}

// ============================================================
//  主程序
// ============================================================
fn main() -> std::io::Result<()> {
    let total_start = Instant::now();
    let prep_start = Instant::now();

    // ===== 优化点1：只获取一次可执行文件路径，不影响后续计算 =====
    let exe_dir = std::env::current_exe()
        .expect("无法获取可执行文件路径")
        .parent()
        .expect("无法获取可执行文件所在目录")
        .to_path_buf();

    let seed_i64 = WORLD_SEED as i64;

    let (min_x, max_x) = (CENTER_X - RADIUS, CENTER_X + RADIUS);
    let (min_z, max_z) = (CENTER_Z - RADIUS, CENTER_Z + RADIUS);
    let x_count = (max_x - min_x + 1) as usize;
    let z_count = (max_z - min_z + 1) as usize;
    let total = x_count * z_count;

    let world_min_x = min_x * 16;
    let world_max_x = max_x * 16 + 15;
    let world_min_z = min_z * 16;
    let world_max_z = max_z * 16 + 15;

    // ===== 优化点2：预计算列宽（只有整数运算，极轻量） =====
    let mut max_w = 1;
    for x in min_x..=max_x {
        max_w = max_w.max(num_width(x));
    }
    for z in min_z..=max_z {
        max_w = max_w.max(num_width(z));
    }
    let cell_w = max_w + 1;
    let z_col_w = max_w + 1;

    // ===== 优化点3：表头预生成（一次性开销） =====
    let x_headers: Vec<Vec<u8>> = (min_x..=max_x)
        .map(|x| format!("{:<w$}", x, w = cell_w).into_bytes())
        .collect();
    let z_headers: Vec<Vec<u8>> = (min_z..=max_z)
        .map(|z| format!("{:<w$}|", z, w = max_w).into_bytes())
        .collect();

    let mut zero_cell = vec![b' '; cell_w];
    zero_cell[0] = b'0';
    let mut one_cell = vec![b' '; cell_w];
    one_cell[0] = b'1';

    // ===== 优化点4：预计算每个坐标的贡献值，避免在循环中重复乘法和溢出检查 =====
    let fx_arr: Vec<i64> = (min_x..=max_x)
        .map(|x| {
            let xi = x as i32;
            let term1 = (xi.wrapping_mul(xi).wrapping_mul(C_X2 as i32)) as i64;
            let term2 = (xi.wrapping_mul(C_X1 as i32)) as i64;
            term1 + term2
        })
        .collect();

    let fz_arr: Vec<i64> = (min_z..=max_z)
        .map(|z| {
            let zi = z as i32;
            let term3 = (zi.wrapping_mul(zi) as i64) * C_Z2;
            let term4 = (zi.wrapping_mul(C_Z1 as i32)) as i64;
            term3 + term4
        })
        .collect();

    let prep_time = prep_start.elapsed();

    // ---------- 计算阶段 ----------
    let compute_start = Instant::now();

    // ========== 1. 生成网格（并行） ==========
    eprintln!("生成 {}×{} 网格...", x_count, z_count);
    let grid_start = Instant::now();
    let mut grid = vec![0u8; total];

    // ===== 优化点5：Rayon并行迭代，每个chunk独立计算，无锁竞争 =====
    grid.par_chunks_mut(x_count)
        .enumerate()
        .for_each(|(z_offset, row)| {
            let fz = fz_arr[z_offset];
            let mut rng = FastRandom { seed: 0 };
            for (x_offset, val) in row.iter_mut().enumerate() {
                let fx = fx_arr[x_offset];
                let combined = (seed_i64 as u64)
                    .wrapping_add(fx as u64)
                    .wrapping_add(fz as u64) as i64;
                rng.set_seed(combined ^ XOR_MASK);
                *val = if rng.next_int_mod(BOUND) == TARGET {
                    1
                } else {
                    0
                };
            }
        });

    let grid_time = grid_start.elapsed();
    eprintln!("网格生成耗时: {:?}", grid_time);

    // ========== 2. 输出种子文件（可选） ==========
    if OUTPUT_SEED_FILE {
        let seed_filename = format!("seed_{}.txt", seed_i64);
        let seed_path = exe_dir.join(&seed_filename);
        output_seed_file(
            min_x, max_x, min_z, max_z,
            x_count, z_count,
            cell_w, z_col_w,
            &x_headers, &z_headers,
            &fx_arr, &fz_arr,
            &zero_cell, &one_cell,
            &seed_path,
            seed_i64,
        )?;
        eprintln!("种子文件已输出 {}", seed_filename);
    }

    let compute_write_time = compute_start.elapsed();

    // ========== 3. 模式匹配 ==========
    let match_start = Instant::now();
    eprintln!("开始模式匹配...");
    let mut matches = Vec::with_capacity(if MATCH_TARGET == 0 {
        1024
    } else {
        MATCH_TARGET
    });

    let max_x_start = x_count - PATTERN_WIDTH + 1;
    let max_z_start = z_count - PATTERN_HEIGHT + 1;

    // ===== 优化点6：使用简单循环进行模式匹配（由于模式全为1，可简化，但保留通用性） =====
    'outer: for z_start in 0..max_z_start {
        for x_start in 0..max_x_start {
            let mut ok = true;
            'check: for pz in 0..PATTERN_HEIGHT {
                for px in 0..PATTERN_WIDTH {
                    let pat = PATTERN[pz][px];
                    if pat == 0 {
                        continue;
                    }
                    let gz = z_start + pz;
                    let gx = x_start + px;
                    let val = grid[gz * x_count + gx];
                    if (pat == 1 && val != 1) || (pat == 2 && val != 0) {
                        ok = false;
                        break 'check;
                    }
                }
            }
            if ok {
                let coord_x = min_x + x_start as i32;
                let coord_z = min_z + z_start as i32;
                matches.push((coord_x, coord_z));
                if MATCH_TARGET > 0 && matches.len() >= MATCH_TARGET {
                    break 'outer;
                }
            }
        }
    }
    eprintln!("匹配耗时: {:?}", match_start.elapsed());

    // ========== 4. 输出匹配结果 ==========
    let match_filename = format!("match_{}.txt", seed_i64);
    let match_path = exe_dir.join(&match_filename);
    let match_file = File::create(&match_path)?;
    let mut writer = BufWriter::with_capacity(1024 * 1024, match_file);

    writeln!(
        writer,
        "==== Minecraft Java Slime Chunk Pattern Match Result ===="
    )?;
    writeln!(writer, "World Seed:            {}", seed_i64)?;
    writeln!(
        writer,
        "Center Chunk X/Z:      {}, {}",
        CENTER_X, CENTER_Z
    )?;
    writeln!(writer, "Scan Radius:           {}", RADIUS)?;
    writeln!(
        writer,
        "X Range [min, max]:    [{}, {}]",
        min_x, max_x
    )?;
    writeln!(
        writer,
        "Z Range [min, max]:    [{}, {}]",
        min_z, max_z
    )?;
    writeln!(
        writer,
        "World X Range:         [{}, {}]",
        world_min_x, world_max_x
    )?;
    writeln!(
        writer,
        "World Z Range:         [{}, {}]",
        world_min_z, world_max_z
    )?;
    writeln!(
        writer,
        "Map Dimensions:        {} × {} chunks",
        x_count, z_count
    )?;
    writeln!(
        writer,
        "Pattern Matrix ({}×{}):",
        PATTERN_HEIGHT, PATTERN_WIDTH
    )?;
    for row in 0..PATTERN_HEIGHT {
        write!(writer, "  ")?;
        for col in 0..PATTERN_WIDTH {
            write!(writer, "{} ", PATTERN[row][col])?;
        }
        writeln!(writer)?;
    }
    writeln!(
        writer,
        "Legend: 0 = any, 1 = must be slime, 2 = must be non-slime"
    )?;
    writeln!(
        writer,
        "Match Target:           {} ({} found)",
        MATCH_TARGET,
        matches.len()
    )?;
    writeln!(writer, "==================================================")?;
    writeln!(
        writer,
        "Found Matches (top-left chunk coordinates, X Z):"
    )?;
    for (x, z) in &matches {
        let wmin_x = x * 16;
        let wmax_x = x * 16 + 15;
        let wmin_z = z * 16;
        let wmax_z = z * 16 + 15;
        writeln!(
            writer,
            "  ({}, {})  ->  world: X [{}, {}], Z [{}, {}]",
            x, z, wmin_x, wmax_x, wmin_z, wmax_z
        )?;
    }
    writer.flush()?;
    eprintln!("匹配结果已写入 {}", match_filename);

    let match_total_time = match_start.elapsed();
    let total_time = total_start.elapsed();

    // ---------- 统计信息 ----------
    println!("\n========== 扫描完成 ==========");
    println!("种子: {}", seed_i64);
    println!("扫描中心: ({}, {})", CENTER_X, CENTER_Z);
    println!("扫描半径: {}", RADIUS);
    println!(
        "区块范围: X [{}, {}], Z [{}, {}]",
        min_x, max_x, min_z, max_z
    );
    println!(
        "世界坐标范围: X [{}, {}], Z [{}, {}]",
        world_min_x, world_max_x, world_min_z, world_max_z
    );
    println!("总区块数: {} × {} = {}", x_count, z_count, total);
    println!("匹配目标: {} 个 (找到 {})", MATCH_TARGET, matches.len());
    println!("预处理耗时: {:?}", prep_time);
    println!("计算+写入耗时: {:?}", compute_write_time);
    println!("匹配耗时: {:?}", match_total_time);
    println!("总耗时: {:?}", total_time);
    println!("输出文件: {}", match_filename);
    if OUTPUT_SEED_FILE {
        let seed_filename = format!("seed_{}.txt", seed_i64);
        let seed_path = exe_dir.join(&seed_filename);
        // 文件已经写入成功，可以直接 canonicalize
        if let Ok(seed_abs) = std::fs::canonicalize(&seed_path) {
            println!("种子文件: {}", seed_filename);
            println!("  绝对路径: {}", seed_abs.display());
        } else {
            // 如果 canonicalize 失败（极少数情况），至少打印文件名
            println!("种子文件: {}", seed_filename);
        }
    }
    eprintln!("匹配结果已写入 {}", match_filename);
    let match_abs = std::fs::canonicalize(&match_path)?;
    println!("  绝对路径: {}", match_abs.display());
    Ok(())
}