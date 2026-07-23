/**
 * 史莱姆区块批量判定程序 (Java版)
 * 输入参数: <坐标文件>
 * 输出: 01字符串表示每个区块是否为史莱姆区块
 */
import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;

public class batch {

    public static boolean isSlimeChunk(long worldSeed, int chunkX, int chunkZ) {
        long part1 = (long) chunkX * chunkX * 4987142L;
        long part2 = (long) chunkX * 5947611L;
        long part3 = (long) chunkZ * chunkZ * 4392871L;
        long part4 = (long) chunkZ * 389711L;

        long seed = worldSeed + part1 + part2 + part3 + part4 ^ 987234911L;

        final long MULTIPLIER = 0x5DEECE66DL;
        final long ADDEND = 0xBL;
        final long MASK = (1L << 48) - 1;

        long rngSeed = (seed ^ MULTIPLIER) & MASK;
        rngSeed = (rngSeed * MULTIPLIER + ADDEND) & MASK;
        int bits = (int) (rngSeed >>> (48 - 31));

        return bits % 10 == 0;
    }

    public static void main(String[] args) {
        if (args.length != 1) {
            System.err.println("用法: java batch <坐标文件>");
            System.exit(1);
        }

        String filename = args[0];

        try (BufferedReader br = new BufferedReader(new FileReader(filename))) {
            String line;
            while ((line = br.readLine()) != null) {
                line = line.trim();
                if (line.isEmpty() || line.startsWith("#")) {
                    continue;
                }
                String[] parts = line.split(",");
                if (parts.length != 3) {
                    continue;
                }
                long seed = Long.parseLong(parts[0]);
                int x = Integer.parseInt(parts[1]);
                int z = Integer.parseInt(parts[2]);

                boolean result = isSlimeChunk(seed, x, z);
                System.out.print(result ? "1" : "0");
            }
            System.out.println();
        } catch (IOException e) {
            System.err.println("读取文件失败: " + e.getMessage());
            System.exit(1);
        }
    }
}
