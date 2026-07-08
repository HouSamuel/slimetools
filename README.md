# slime_chunk_tools

Minecraft Java 版史莱姆区块扫描工具，vibecoding。

## 怎么用

所有参数都在 `src/config.rs` 里改，改完直接编译运行就行，`main.rs` 不用碰。

```rust
pub const CONFIG: Config = Config {
    world_seed: 20260627,      // 世界种子，随便填
    center_block_x: 0,         // 中心区块 X 坐标
    center_block_z: 0,         // 中心区块 Z 坐标
    radius: 100,               // 扫描半径（区块）
    
    mode: Mode::Check,         // Check / Match / Count 三种模式
    
    // Match 模式用的
    pattern: Some(PATTERN_ROWS),
    match_target: 0,           // 0 = 找全部，其他数字 = 找到就停
    
    // Count 模式用的
    count_shape: CountShape::Square,    // 正方形或圆形
    count_size: 5,             // 区域大小
    count_target: 10,          // 取前几个
    
    memory_limit_gib: 4.0,     // 内存限制，超过会自动分块
    
    // 输出开关
    output_map: true,          // 输出网格图
    output_match: true,        // 输出匹配结果（Match模式）
    output_count: true,        // 输出计数结果（Count模式）
    output_log: true,          // 输出日志文件
    
    terminal_mode: TerminalMode::Basic,
    
    // 安全模式，防止生成太大的 map 文件
    secure_mode: false,
    challenge_code: None,
    
    progress_update_interval: 1.0,
};
```

## 三种运行模式

### Check（检测）

扫描区域内所有史莱姆区块，输出网格图。

### Match（匹配）

按图案找史莱姆区块分布：
- 0 = 任意区块
- 1 = 必须是史莱姆区块
- 2 = 必须不是史莱姆区块

### Count（计数）

在指定形状的区域内统计史莱姆区块数量，取史莱姆最多的前 N 个区域。

## 输出文件

- `{seed}_map.txt` - 网格图，0 和 1 的矩阵，对齐显示
- `{seed}_match.txt` - 匹配结果（Match 模式）
- `{seed}_count.txt` - 计数结果（Count 模式）
- `{seed}_log_{时间戳}.log` - 完整日志，带时间戳

## 安全模式

如果开启了 `secure_mode`，且预估 map 文件大于 5GiB，程序会要求输入挑战码。挑战码每 10 分钟变一次，运行程序会自动告诉你当前的挑战码，复制到 `challenge_code` 里就行。

## 运行

```bash
cargo run --release
```

## 模块说明

- **config** - 配置参数，纯定义
- **preprocess** - 参数校验和预处理
- **slime_chunk** - 史莱姆区块判定核心
- **grid** - 网格生成
- **matcher** - 图案匹配
- **counter** - 区域计数
- **grid_output / match_output / count_output** - 文件输出
- **terminal** - 终端输出
- **log** - 日志文件
