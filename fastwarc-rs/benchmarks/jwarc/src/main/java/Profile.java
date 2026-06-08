import org.netpreserve.jwarc.WarcReader;
import org.netpreserve.jwarc.WarcRecord;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.file.Path;
import java.nio.file.Paths;

public class Profile {
    public static void main(String[] args) throws IOException {
        if (args.length != 1) {
            System.err.printf("Usage: %s WARCFILE%n", System.getProperty("sun.java.command"));
            System.exit(1);
        }

        Path path = Paths.get(args[0]);
        long start = System.nanoTime();
        long lastTimer = start;
        long lastCount = 0;
        long lastBytes = 0;
        long totalCount = 0;
        long totalBytes = 0;

        System.out.println("Reading WARC file: " + path);
        ByteBuffer buffer = ByteBuffer.allocate(4096 << 10);
        buffer.flip();
        try (WarcReader reader = new WarcReader(FileChannel.open(path), buffer)) {
            for (WarcRecord record : reader) {
                long contentLength = record.body().size();
                if (contentLength < 0) {
                    contentLength = 0;
                }

                lastCount += 1;
                lastBytes += contentLength;
                totalCount += 1;
                totalBytes += contentLength;

                long now = System.nanoTime();
                double elapsed = (now - lastTimer) / 1_000_000_000.0;
                if (elapsed >= 0.5) {
                    System.out.printf(
                            "%.0f records/s, %.1f MiB/s, %.1f KiB/rec (%d total, %.1f MiB)%n",
                            lastCount / elapsed,
                            lastBytes / elapsed / 1024.0 / 1024.0,
                            lastBytes / (double) Math.max(lastCount, 1) / 1024.0,
                            totalCount,
                            totalBytes / 1024.0 / 1024.0
                    );
                    lastCount = 0;
                    lastBytes = 0;
                    lastTimer = now;
                }
            }
        }
        System.out.printf("Time elapsed: %.1fs%n", (System.nanoTime() - start) / 1_000_000_000.0);
    }
}
