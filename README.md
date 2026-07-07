# slime_chunk_tools
这是一个用于处理我的世界java版史莱姆区块的工具(vibecoding产物)

由rust编写
### 史莱姆区块判定
slime_check.rs

开头硬编码配置区
```rust
const WORLD_SEED: i128 = 20260627;//世界种子，非法种子需要自己在wiki上转换
const CENTER_X: i32 = 0;//开始查找的区块的x坐标，是区块坐标不是世界坐标
const CENTER_Z: i32 = 0;//开始查找的区块的z坐标，也是区块坐标不是世界坐标
const RADIUS: i32 = 100;//向外拓展的半径，单位是区块
//现在就是以0,0这个区块为中心，向外拓展100个区块，这是一个正方形，作为检查区域

/// 是否输出完整种子文件（0/1 矩阵）：true = 输出，false = 不输出
const OUTPUT_SEED_FILE: bool = true;//不知道在为什么增加了这个功能，就默认true吧
```
### 史莱姆区块匹配
slime_check_match.rs

开头也是硬编码配置
```rust
const WORLD_SEED: i128 = 20260627;
const CENTER_X: i32 = 0;
const CENTER_Z: i32 = 0;
const RADIUS: i32 = 100;
//这些都一样的
/// 匹配模式矩阵（二维，行优先）
/// 0 = 任意（不检查）
/// 1 = 必须为史莱姆区块
/// 2 = 必须为非史莱姆区块
const PATTERN: [[u8; 3]; 3] = [//这里更改矩阵时两个数字也许要改，对应矩阵长宽
    [2, 1, 2],
    [2, 1, 2],
    [1, 2, 1],
];
const PATTERN_WIDTH: usize = 3;//这里也许要改数字
const PATTERN_HEIGHT: usize = 3;//这里也许要改数字

/// 匹配目标：0 = 找出所有匹配，1 = 找到一个即停止，2 = 找到两个即停止
const MATCH_TARGET: usize = 0;

/// 是否输出完整种子文件（0/1 矩阵）：true = 输出，false = 不输出
const OUTPUT_SEED_FILE: bool = true;
```
---
想要哪个就把对应文件的代码复制到main.rs中

这些都是通过硬编码设置，在目录中运行`cargo run --release`编译后运行

输出的文件我想应该足够详细，可以直接读的，这里有输出示例

终端输出示例
```shell
   Compiling slime_chunk_scanner v0.1.0 (/Users/qqhou/Desktop/tools/slime_chunk_tool)
    Finished `release` profile [optimized] target(s) in 2.89s
     Running `target/release/slime_chunk_scanner`
生成 201×201 网格...
网格生成耗时: 288.708µs
种子文件已输出 seed_20260627.txt
开始模式匹配...
匹配耗时: 115µs
匹配结果已写入 match_20260627.txt

========== 扫描完成 ==========
种子: 20260627
扫描中心: (0, 0)
扫描半径: 100
区块范围: X [-100, 100], Z [-100, 100]
世界坐标范围: X [-1600, 1615], Z [-1600, 1615]
总区块数: 201 × 201 = 40401
匹配目标: 0 个 (找到 2)
预处理耗时: 116.083µs
计算+写入耗时: 1.151291ms
匹配耗时: 290.291µs
总耗时: 1.558ms
输出文件: match_20260627.txt
种子文件: seed_20260627.txt
  绝对路径: /....../slime_chunk_tool/target/release/seed_20260627.txt
匹配结果已写入 match_20260627.txt
  绝对路径: /....../slime_chunk_tool/target/release/match_20260627.txt
```
文件输出示例:

输出种子文件：[seed_20260627.txt](seed_20260627.txt)

输出匹配文件：[match_20260627.txt](match_20260627.txt)
