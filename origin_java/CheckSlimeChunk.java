import java.util.Random;

public class CheckSlimeChunk {

    public static boolean isSlimeChunk(long worldSeed, int chunkX, int chunkZ) {
        Random rng = new Random(
                worldSeed +
                        (long) (chunkX * chunkX * 4987142) +
                        (long) (chunkX * 5947611) +
                        (long) (chunkZ * chunkZ) * 4392871L +
                        (long) (chunkZ * 389711) ^ 987234911L
        );
        return rng.nextInt(10) == 0;
    }

    public static void main(String[] args) {
        // 校验命令行参数数量
        if (args.length != 3) {
            System.out.println("使用方式：java CheckSlimeChunk <世界种子> <区块X> <区块Z>");
            System.out.println("示例：java CheckSlimeChunk 123456 -2 5");
            System.out.println("说明：");
            System.out.println("  1. 世界种子：/seed 获取的长数字，支持负数");
            System.out.println("  2. chunkX / chunkZ：区块坐标，玩家坐标÷16向下取整");
            return;
        }

        long seed;
        int cx, cz;
        try {
            // 解析命令行传入的三个参数
            seed = Long.parseLong(args[0]);
            cx = Integer.parseInt(args[1]);
            cz = Integer.parseInt(args[2]);
        } catch (NumberFormatException e) {
            System.out.println("错误：参数必须是数字，种子为长整数，区块坐标为整数");
            return;
        }

        boolean isSlime = isSlimeChunk(seed, cx, cz);
        System.out.println("=== result ===");
        System.out.println("world seed：" + seed);
        System.out.println("chunk position X = " + cx + " , Z = " + cz);
        System.out.println(isSlime ? "True" : "False");
    }
}