public class VerifySlimeChunk {
    private static final long C_X2 = 4987142L;
    private static final long C_X1 = 5947611L;
    private static final long C_Z2 = 4392871L;
    private static final long C_Z1 = 389711L;
    private static final long XOR_SEED = 0x5DEECE66DL;
    private static final long LCG_MASK = (1L << 48) - 1;
    
    public static boolean isSlimeChunk(long seed, int chunkX, int chunkZ) {
        long fx = (long) chunkX * chunkX * C_X2 + (long) chunkX * C_X1;
        long fz = (long) chunkZ * chunkZ * C_Z2 + (long) chunkZ * C_Z1;
        long combined = seed ^ fx ^ fz;
        
        long lcgSeed = (combined ^ XOR_SEED) & LCG_MASK;
        lcgSeed = (lcgSeed * 0x5DEECE66DL + 0xBL) & LCG_MASK;
        int bits = (int) (lcgSeed >> 17);
        
        return bits % 10 == 0;
    }
    
    public static void main(String[] args) {
        long seed = 20260627L;
        
        System.out.println("=== 验证输出坐标 ===");
        int[][] coords = {
            {2155, 12308},
            {-13417, 6837},
            {-35583, -7392},
            {-38278, 13512}
        };
        
        for (int[] coord : coords) {
            int x = coord[0];
            int z = coord[1];
            boolean slime = isSlimeChunk(seed, x, z);
            System.out.printf("区块(%5d, %5d): %s%n", x, z, slime ? "史莱姆" : "普通");
        }
        
        System.out.println("\n=== 验证 3x3 区域 ===");
        int testX = 2155;
        int testZ = 12308;
        
        for (int dz = 0; dz < 3; dz++) {
            for (int dx = 0; dx < 3; dx++) {
                int x = testX + dx;
                int z = testZ + dz;
                boolean slime = isSlimeChunk(seed, x, z);
                System.out.print(slime ? "1 " : "0 ");
            }
            System.out.println();
        }
        
        System.out.println("\n=== 验证正确算法 ===");
        int correctCount = 0;
        for (int dz = 0; dz < 3; dz++) {
            for (int dx = 0; dx < 3; dx++) {
                int x = testX + dx;
                int z = testZ + dz;
                boolean slime = isSlimeChunk(seed, x, z);
                if (slime) correctCount++;
            }
        }
        System.out.printf("3x3 区域中史莱姆区块数: %d/9%n", correctCount);
    }
}