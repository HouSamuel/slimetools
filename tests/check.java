/**
 * 史莱姆区块判定程序 (Java版)
 * 输入参数: <seed> <chunkX> <chunkZ>
 * 输出: true 或 false
 */
public class check {

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
        if (args.length != 3) {
            System.err.println("用法: java check <seed> <chunkX> <chunkZ>");
            System.exit(1);
        }

        long seed = Long.parseLong(args[0]);
        int chunkX = Integer.parseInt(args[1]);
        int chunkZ = Integer.parseInt(args[2]);

        boolean result = isSlimeChunk(seed, chunkX, chunkZ);
        System.out.println(result ? "true" : "false");
    }
}
