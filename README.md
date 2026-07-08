# slime_chunk_tools

Minecraft Java 版史莱姆区块扫描工具。

## 配置

所有参数都在 `src/config.rs` 里设置，改完直接编译运行就行，`main.rs` 不用动。

```rust
// 匹配图案，改这个矩阵就行
const PATTERN_ROWS: &[&[u8]] = &[
    &[1, 1],
    &[1, 1],
];

pub const CONFIG: Config = Config {
    world_seed: 20260627,      // 世界种子
    center_block_x: 0,         // 中心区块 X
    center_block_z: 0,         // 中心区块 Z
    radius: 100,               // 扫描半径（区块）
    
    mode: Mode::Match,         // Check / Match / Count
    
    pattern: Some(Pattern::new(...)),  // 匹配图案（Match模式用）
    match_target: 0,           // 0 = 全部查找，其他值 = 找到指定数量停止
    
    count_shape: CountShape::Square,    // Square / Circle（Count模式用）
    count_size: 5,             // 区域尺寸（Count模式用）
    count_target: 10,          // 取前多少个（Count模式用）
    
    memory_limit_gib: None,    // 内存限制，单位 GiB（可选）
    
    generate_seed_file: false, // 是否生成网格图
    terminal_output: true,     // 终端输出开关
    log_output: true,          // 日志文件开关
};
```

### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| world_seed | i128 | 世界种子 |
| center_block_x | i32 | 中心区块 X 坐标 |
| center_block_z | i32 | 中心区块 Z 坐标 |
| radius | i32 | 扫描半径，以中心区块为原点向四周扩展 |
| mode | Mode | 运行模式：Check/Match/Count |
| pattern | Option\<Pattern\> | 匹配图案，0=任意，1=史莱姆，2=普通 |
| match_target | usize | Match模式：0=全部查找，其他值=找到即停止 |
| count_shape | CountShape | Count模式：Square=正方形，Circle=圆形 |
| count_size | i32 | Count模式：正方形边长或圆形半径 |
| count_target | usize | Count模式：取前多少个区域 |
| memory_limit_gib | Option\<f64\> | 内存限制，达到后自动分块处理 |
| generate_seed_file | bool | 是否生成网格可视化文件 |
| terminal_output | bool | 是否在终端输出进度和统计 |
| log_output | bool | 是否生成日志文件 |

## 运行模式

### Check（检测）

扫描区域内所有史莱姆区块，可选输出网格文件。

### Match（匹配）

按图案匹配史莱姆区块分布：
- 0 = 任意区块
- 1 = 必须是史莱姆区块
- 2 = 必须不是史莱姆区块

### Count（计数）

在指定形状的区域内统计史莱姆区块数量，取前 N 个最多的区域。

## 输出文件

- `{seed}_map.txt` - 网格可视化（需开启 generate_seed_file）
- `{seed}_match.txt` - 匹配结果（Match 模式）
- `{seed}_count.txt` - 计数结果（Count 模式）
- `{seed}_log.txt` - 完整日志

## 运行

```bash
cargo run --release
```

## 模块说明

- **config** - 纯参数定义，不包含校验逻辑
- **preprocess** - 输入校验和数据预处理，确保参数合法并提前计算常用值
- **slime_chunk** - 史莱姆区块判定核心逻辑
- **grid** - 内存网格生成，追求效率
- **matcher** - 图案匹配算法
- **counter** - 区域计数统计
- **grid_output / match_output / count_output** - 文件输出，格式化美化
- **stats** - 运行耗时统计
- **terminal** - 终端输出控制
- **log** - 日志文件输出