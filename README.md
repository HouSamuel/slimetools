# Slimetools - 高性能史莱姆区块图案匹配系统

在 Minecraft 极大规模地图中，极速搜索所有符合特定"二维图案"的史莱姆区块坐标。采用极致的位运算（SWAR）和多线程数据并行，将计算效率推向单核理论极限。

## ✨ 功能特性

- **极致性能**：扫描效率高达 **19.8 亿区块/秒**（400 亿区块仅需 20 秒）
- **SWAR 位运算**：将连续 64 个区块打包成 u64，一次性比较多个位置
- **多线程并行**：按 X 轴切分任务，充分利用多核 CPU
- **零冗余计算**：倒置循环轴 + 环形缓存，每个区块只计算一次
- **智能短路**：按条件严格程度重排行顺序，快速淘汰无效坐标
- **精确匹配**：完全复现 Java 版 Minecraft 的史莱姆区块算法
- **灵活配置**：支持自定义图案、扫描范围、输出模式

## 🚀 快速开始

### 编译

```bash
# 编译发布版本（推荐，开启所有优化）
cargo build --release

# 编译调试版本
cargo build
```

### 运行

```bash
# 使用默认配置
./target/release/slimetools

# 指定配置文件
./target/release/slimetools config.toml
```

## ⚙️ 配置说明

编辑 `config.toml` 文件：

```toml
# 世界种子（i64 类型）
seed = 1234567890

# 中心区块坐标
center_x = 0
center_z = 0

# 扫描半径（区块数），实际范围为 [-radius, radius]
radius = 10000

# 匹配图案
# 1: 必须是史莱姆区块
# 0: 必须不是史莱姆区块
# 2: 任意（忽略）
pattern = [
    [1, 1, 1],
    [1, 1, 1],
    [1, 1, 1]
]

# 最大匹配数（0 表示无限制）
limit = 0

# 输出模式：console / file / both
output_mode = "both"

# 输出文件路径（file 和 both 模式有效，会被自动覆盖为种子号命名）
output_path = "matches.txt"
```

## 🧠 核心算法

### 史莱姆区块判定

完全复现 Java 版 Minecraft 的史莱姆区块算法：

```rust
#[inline(always)]
pub fn is_slime_chunk(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let cx = chunk_x as i64;
    let cz = chunk_z as i64;
    
    let part1 = cx * cx * 4987142i64;
    let part2 = cx * 5947611i64;
    let part3 = cz * cz * 4392871i64;
    let part4 = cz * 389711i64;
    
    let seed = world_seed
        .wrapping_add(part1)
        .wrapping_add(part2)
        .wrapping_add(part3)
        .wrapping_add(part4)
        ^ 987234911i64;
    
    // Java Random LCG 算法
    const MULTIPLIER: i64 = 0x5DEECE66D;
    const ADDEND: i64 = 0xB;
    const MASK: i64 = (1i64 << 48) - 1;
    
    let mut rng_seed = (seed ^ MULTIPLIER) & MASK;
    rng_seed = (rng_seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND)) & MASK;
    let bits = (rng_seed >> (48 - 31)) as i32;
    
    bits % 10 == 0
}
```

### 性能优化策略

| 优化策略 | 原理 | 效果 |
|----------|------|------|
| **位图打包** | 将连续 64 个区块打包成 u64 | 单次位运算比较 64 个位置 |
| **边界无缝拼接** | 使用 u128 拼接当前块和下一块 | 完美解决跨 64 位边界问题 |
| **倒置循环轴** | 外层 X 步进 64，内层 Z 步进 1 | 配合环形缓存实现 O(N) 零冗余 |
| **环形缓存** | 维护 H 行数据，每个区块只计算一次 | 消除 O(N×H) 的重复计算 |
| **行短路重排** | 按条件数重排行顺序 | 提升短路命中率 50%+ |
| **多线程并行** | 按 X 轴切分任务 | 充分利用多核 CPU |
| **提前短路** | 内层循环中 combined==0 立即 break | 节省无用位运算 |

## 📁 项目结构

```
slimetools/
├── src/
│   ├── lib.rs              # 库入口，导出 slime_lib 模块
│   ├── main.rs             # 主程序，作为数据中转站
│   ├── slime_lib.rs        # 史莱姆区块判定核心算法
│   ├── pattern.rs          # 图案预处理（行重排优化）
│   ├── preprocess.rs       # 配置文件解析和预处理
│   ├── matcher.rs          # 匹配核心逻辑（SWAR + 多线程）
│   ├── matcher_output.rs   # 匹配结果格式化输出
│   ├── info.rs             # 运行信息格式化输出
│   ├── timer.rs            # 计时模块
│   └── stats.rs            # 统计信息生成
├── tests/
│   ├── slime_test.rs       # 集成测试（check/generate/batch/compare）
│   ├── check.java          # Java 单区块判定程序
│   └── batch.java          # Java 批量判定程序
├── config.toml             # 默认配置文件
├── Cargo.toml              # 项目依赖配置
└── README.md               # 项目文档
```

## 📊 输出示例

```
[Info]
世界种子: 1234567890
中心区块坐标: (0, 0)
世界坐标: x:[0, 16) z:[0, 16)
扫描半径: 100000 chunks
区块坐标范围: x:(-100000, 100000) z:(-100000, 100000)
世界坐标范围: x:[-1600000, 1600048) z:[-1600000, 1600048)
图案:
行 1: 1 1 1
行 2: 1 1 1
行 3: 1 1 1
[Result]
找到 54 个匹配（按距离中心排序）：

[ 序/总数]  (   区块X,    区块Z)  =>  x:[     世界X,     世界X+) z:[     世界Z,     世界Z+)  距离:     距离 区块
--------------------------------------------------------------------------------------------
[ 1/54]  (  2565,  11761)  =>  x:[   41040,    41056) z:[  188176,   188192)  距离:  14326 区块
...
[Statistics]
预处理耗时: 493.33µs
计算耗时: 20.19s
输出耗时: 214.92µs
总耗时: 20.19s
扫描区块总数: 40,000,400,001
计算效率: 1,980,860,424.72 区块/秒
匹配结果数: 54
```

## 🧪 测试

### Rust 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --test slime_test -- --nocapture

# 单区块判定测试
cargo test --test slime_test -- --nocapture -- check 1234567890 0 0

# 生成随机测试数据
cargo test --test slime_test -- --nocapture -- generate 1000 tests/chunks.txt

# 批量判定（Rust版）
cargo test --test slime_test -- --nocapture -- batch tests/chunks.txt

# 对比测试（Rust vs Java）
cargo test --test slime_test -- --nocapture -- compare tests/chunks.txt
```

### Java 测试

```bash
# 编译 Java 程序
javac tests/check.java
javac tests/batch.java

# 单区块判定
java -cp tests check <seed> <chunkX> <chunkZ>

# 批量判定
java -cp tests batch <坐标文件>
```

## 🏆 性能对比

| 版本 | 算法 | 效率 |
|------|------|------|
| v1.0 | 逐块判定 | ~7200 万区块/秒 |
| v2.0 | SWAR + 多线程 | ~2.06 亿区块/秒 |
| v3.0 | 倒置循环 + 环形缓存 + 行重排 | **~19.8 亿区块/秒** |

## 📝 注意事项

1. **图案限制**：图案宽度和高度不能超过 64
2. **内存使用**：算法采用即时计算策略，内存占用极低（O(1)）
3. **Java 运行环境**：对比测试需要系统已安装 JDK
4. **大半径扫描**：半径超过 100000 时需要较长时间，请耐心等待

## 📄 许可证

MIT License

---

## 🔧 Developer Notes（开发者笔记）

### 实验性算法开关

`src/matcher.rs` 文件顶部包含一个隐藏的实验性算法开关：

```rust
const EXPERIMENTAL_REVERSE_JUMP_MODE: bool = false;
```

**功能说明**：
- 当设置为 `true` 时，启用**逆向扫描 + 废行跳跃算法**
- 该算法通过逆向遍历 Z 轴，识别"废行"并跳跃跳过，在大图案（5x5以上）搜索中可能获得显著性能提升
- **默认关闭**，普通用户不应修改此值

**风险警告**：
- 此算法为实验性，在小图案（如 3x3）上可能导致性能下降或结果错误
- 严禁将此选项暴露给用户配置文件
- 使用前请确保已通过所有测试用例
