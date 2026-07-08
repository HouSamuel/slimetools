public class TestSlimeChunk {
    private static final long C_X2 = 4987142L;
    private static final long C_X1 = 5947611L;
    private static final long C_Z2 = 4392871L;
    private static final long C_Z1 = 389711L;
    
    public static boolean isSlimeChunk(long seed, int chunkX, int chunkZ) {
        long fx = (long) chunkX * chunkX * C_X2 + (long) chunkX * C_X1;
        long fz = (long) chunkZ * chunkZ * C_Z2 + (long) chunkZ * C_Z1;
        
        long combined = seed ^ fx ^ fz;
        
        java.util.Random random = new java.util.Random(combined);
        return random.nextInt(10) == 0;
    }
    
    public static void main(String[] args) {
        long seed = 20260627L;
        
        System.out.println("=== Finding 3x3 Full Slime Patterns ===");
        System.out.println("Seed: " + seed);
        System.out.println();
        
        // 在小范围内搜索真正的 3x3 全史莱姆区块图案
        int found = 0;
        for (int x = -100; x <= 100; x++) {
            for (int z = -100; z <= 100; z++) {
                boolean allSlime = true;
                for (int dz = 0; dz < 3; dz++) {
                    for (int dx = 0; dx < 3; dx++) {
                        if (!isSlimeChunk(seed, x + dx, z + dz)) {
                            allSlime = false;
                            break;
                        }
                    }
                    if (!allSlime) break;
                }
                if (allSlime) {
                    found++;
                    System.out.println("Found 3x3 pattern at (" + x + ", " + z + ")");
                }
            }
        }
        
        System.out.println();
        System.out.println("Total found: " + found);
        
        System.out.println();
        System.out.println("=== Verify specific coordinates ===");
        
        // 验证修复后的算法是否正确
        int[][] testCoords = {
            {-37, 8}, {-36, 8}, {-35, 8},
            {-37, 9}, {-36, 9}, {-35, 9},
            {-37, 10}, {-36, 10}, {-35, 10}
        };
        
        System.out.println("3x3 at (-37, 8):");
        for (int i = 0; i < 9; i++) {
            boolean result = isSlimeChunk(seed, testCoords[i][0], testCoords[i][1]);
            System.out.print(result ? "1 " : "0 ");
            if ((i + 1) % 3 == 0) System.out.println();
        }
    }
}