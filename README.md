# Slimetools — 史莱姆区块图案匹配工具

[![Rust](https://img.shields.io/badge/rust-1.80%2B-blue)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

**Slimetools** 是一个高效扫描 Minecraft 世界中史莱姆区块并匹配自定义图案的命令行工具。它基于 Java 版原版史莱姆区块算法，支持大范围扫描、图案匹配、性能统计和多输出模式。

---

## 特性

- ✅ **精确算法**：完全复刻 Minecraft Java 版的 `Random` 实现，结果与原版一致。
- ✅ **自定义图案**：支持任意尺寸矩形图案，以 `0`（普通区块）、`1`（史莱姆区块）、`2`（任意）定义。
- ✅ **大范围扫描**：以指定中心向外扩展半径，自动匹配所有可能位置。
- ✅ **内存友好**：采用即时计算策略，按需判断区块状态，不预先缓存，适合大规模扫描。
- ✅ **性能统计**：输出预处理、计算、输出三阶段耗时、总扫描区块数、平均效率（区块/秒）和匹配数量。
- ✅ **双输出模式**：可同时输出到控制台和文件（`console`、`file`、`both`）。
- ✅ **自动命名**：输出文件自动以种子号命名（格式：`{seed}_match_output.txt`）。
- ✅ **友好信息**：显示种子、坐标范围、图案等详细信息，世界坐标采用半开区间 `[start, end)`。

---

## 快速开始

### 前置要求

- Rust 1.80+（[安装指南](https://www.rust-lang.org/learn/get-started)）

### 克隆项目

```bash
git clone https://github.com/yourusername/slimetools.git
cd slimetools
```

### 配置 `config.toml`

在项目根目录创建或编辑 `config.toml`：

```toml
seed = 1234567890          # 世界种子（/seed 获取）
center_x = 0               # 扫描中心区块 X
center_z = 0               # 扫描中心区块 Z
radius = 10                # 半径（chunks），向四方扩展
pattern = [                # 图案，行优先
    [1, 2],                # 1=史莱姆, 0=普通, 2=任意
    [1, 0]
]
limit = 0                  # 匹配上限，0 表示无限制
output_mode = "both"       # "console" | "file" | "both"
```

### 构建与运行

```bash
# 开发模式（较快编译）
cargo run

# 发布模式（最大性能）
cargo run --release
```

### 输出示例

```
[Info]
世界种子: 1234567890
中心区块坐标: (0, 0)
世界坐标: x:[0, 16) z:[0, 16)
扫描半径: 10 chunks
区块坐标范围: x:(-10, 10) z:(-10, 10)
世界坐标范围: x:[-160, 192) z:[-160, 192)
图案:
行 1: 1 2
行 2: 1 0

[Result]
找到 4 个匹配：
起始区块: (-10, 6)
起始区块: (-5, 8)
起始区块: (4, -5)
起始区块: (5, 8)

[Statistics]
预处理耗时: 577.96µs
计算耗时: 6.12µs
输出耗时: 668.71µs
总耗时: 1.28ms
扫描区块总数: 484
平均效率: 79020408.16 区块/秒
匹配结果数: 4
```

---

## 配置文件详解

| 字段 | 类型 | 说明 |
|------|------|------|
| `seed` | 64位整数 | 世界种子，与 `/seed` 一致 |
| `center_x`, `center_z` | 32位整数 | 扫描中心区块坐标 |
| `radius` | 非负整数 | 向外扩展的区块数（包含边界），扫描范围为 `(-radius, radius)` |
| `pattern` | 二维整数数组 | 图案定义，每行长度必须一致，值仅为 `0`、`1`、`2` |
| `limit` | 非负整数 | 最大输出匹配数，`0` 表示全部输出 |
| `output_mode` | 字符串 | `"console"`、`"file"` 或 `"both"` |
| `output_path` | 字符串（可选） | 输出文件路径，**已废弃**，现自动使用种子号命名 |

---

## 实现方法

### 史莱姆区块判定算法

史莱姆区块判定基于 Minecraft Java 版的伪随机数生成器实现：

1. **种子计算**：将世界种子与区块坐标混合，生成初始种子：
   ```
   seed = worldSeed + chunkX² × 4987142 + chunkX × 5947611 + chunkZ² × 4392871 + chunkZ × 389711 XOR 987234911
   ```

2. **LCG 随机数生成**：使用线性同余生成器（LCG）生成随机数：
   - 乘数：`0x5DEECE66D`（25214903917）
   - 增量：`0xB`（11）
   - 掩码：`(1 << 48) - 1`（48位）

3. **判定**：生成 0-9 的随机数，若结果为 0 则为史莱姆区块（概率 10%）。

### 图案匹配策略

采用**即时计算策略**，而非预先缓存所有区块结果：

1. **遍历起始位置**：按顺序遍历所有可能的图案起始位置。
2. **逐块验证**：对于每个起始位置，逐个验证图案中的每个位置。
3. **提前终止**：一旦发现不匹配的位置，立即跳过当前起始位置，继续下一个。
4. **内存优化**：无需预先存储所有区块的判定结果，内存使用为 O(1)（除结果列表外）。

这种策略在大规模扫描时具有显著优势，避免了内存不足的问题。

### 坐标范围计算

扫描范围以中心坐标为基准，对称扩展：

- **匹配起始范围**：`x ∈ [center_x - radius, center_x + radius]`，`z ∈ [center_z - radius, center_z + radius]`
- **缓存扩展范围**：为容纳完整图案，缓存范围扩展至 `[x_start, x_end + pattern_width - 1]`

---

## 项目结构

```
slimetools/
├── Cargo.toml           # 项目清单
├── config.toml          # 用户配置
├── README.md            # 本文档
├── origin_java/         # Java 原版算法参考实现
│   ├── CheckSlimeChunk.java
│   └── CheckSlimeChunk.class
└── src/
    ├── main.rs          # 入口，数据中转站，协调各模块
    ├── slime_lib.rs     # 史莱姆区块判断核心算法
    ├── preprocess.rs    # 配置加载、校验与范围计算
    ├── matcher.rs       # 图案匹配引擎（即时计算）
    ├── matcher_output.rs # 格式化匹配结果为字符串
    ├── info.rs          # 生成运行信息字符串
    ├── timer.rs         # 计时器模块，测量各阶段耗时
    └── stats.rs         # 统计模块，生成性能统计信息
```

---

## 模块职责

| 模块 | 职责 | 输入 | 输出 |
|------|------|------|------|
| `main.rs` | **数据中转站**，协调各模块执行流程 | 无（读取配置文件路径） | 控制台输出、文件输出 |
| `slime_lib.rs` | 史莱姆区块判定核心算法，完全复现 Java 版行为 | 世界种子、区块坐标 (x, z) | `bool`（是否为史莱姆区块） |
| `preprocess.rs` | 读取并验证配置文件，计算扫描范围和缓存扩展范围 | `config.toml` 文件路径 | `Preprocessed` 结构体 |
| `matcher.rs` | 图案匹配引擎，遍历所有起始位置并逐块验证 | 预处理结果 | 匹配位置列表、扫描区块总数 |
| `matcher_output.rs` | 格式化匹配结果为可读字符串 | 匹配位置列表 | 格式化字符串（`[Result]` 开头） |
| `info.rs` | 生成运行信息字符串（种子、坐标范围、图案等） | 预处理结果 | 格式化信息字符串（`[Info]` 开头） |
| `timer.rs` | 计时模块，测量预处理、计算、输出各阶段耗时 | 无 | `StageTimes` 结构体 |
| `stats.rs` | 统计模块，生成性能统计信息字符串 | 计时数据、区块数、匹配数 | 格式化统计字符串（`[Statistics]` 开头） |

---

## 执行流程

```
main.rs (中转站)
    │
    ├─→ preprocess.rs (读取配置)
    │       └─→ 返回 Preprocessed 结构体
    │
    ├─→ matcher.rs (执行计算)
    │       ├─→ 调用 slime_lib.rs 逐块判断
    │       └─→ 返回 (匹配列表, 区块总数)
    │
    ├─→ matcher_output.rs (格式化结果)
    │       └─→ 返回 "[Result]..." 字符串
    │
    ├─→ info.rs (生成信息)
    │       └─→ 返回 "[Info]..." 字符串
    │
    ├─→ main.rs (合并输出)
    │       └─→ 输出到控制台/文件
    │
    └─→ stats.rs (生成统计)
            ├─→ 调用 timer.rs 获取耗时
            └─→ 返回 "[Statistics]..." 字符串
```

---

## 性能说明

- **即时计算策略**：不预先缓存所有区块结果，按需判断，内存使用为 O(1)，适合大规模扫描。
- **提前终止优化**：一旦发现不匹配位置立即跳过，避免无效计算。
- **编译优化**：建议使用 `--release` 构建以获得最佳性能。
- **效率指标**：在标准配置下，每秒可扫描约 8000 万区块。

---

## 贡献指南

欢迎提交 Issue 和 Pull Request。请确保代码风格符合 `rustfmt`，并通过 `cargo build`。

---

## 许可

本项目采用 MIT 许可证，详情见 [LICENSE](LICENSE) 文件。

---

## 致谢

- Minecraft 史莱姆区块算法源自 [Minecraft Wiki](https://minecraft.fandom.com/wiki/Slime)。
- Rust 生态提供了强大的性能和生产力。

---

**Happy slime hunting! 🟢**
