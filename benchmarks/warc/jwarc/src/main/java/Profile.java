import org.netpreserve.jwarc.WarcReader;
import org.netpreserve.jwarc.WarcRecord;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.file.Path;
import java.nio.file.Paths;

public class Profile {
    private static final int DEFAULT_BUFFER_SIZE = 4096 << 10;

    private static int bufferSize() {
        String value = System.getenv("BUFFER_SIZE");
        if (value == null) {
            return DEFAULT_BUFFER_SIZE;
        }
        try {
            int parsed = Integer.parseInt(value);
            return parsed > 0 ? parsed : DEFAULT_BUFFER_SIZE;
        } catch (NumberFormatException e) {
            return DEFAULT_BUFFER_SIZE;
        }
    }

    public static void main(String[] args) throws IOException {
        if (args.length != 1) {
            System.err.printf("Usage: %s WARCFILE%n", System.getProperty("sun.java.command"));
            System.exit(1);
        }

        Path path = Paths.get(args[0]);
        int bufferSize = bufferSize();
        long start = System.nanoTime();
        long lastTimer = start;
        long lastCount = 0;
        long lastBytes = 0;
        long totalCount = 0;
        long totalBytes = 0;

        System.out.println("Reading WARC file: " + path);
        ByteBuffer buffer = ByteBuffer.allocate(bufferSize);
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
        double totalElapsed = (System.nanoTime() - start) / 1_000_000_000.0;
        System.out.printf(
                "Summary: %.1fs, %.0f records/s, %.1f MiB/s, %.1f KiB/rec (%d total, %.1f MiB)%n",
                totalElapsed,
                totalCount / totalElapsed,
                totalBytes / totalElapsed / 1024.0 / 1024.0,
                totalBytes / (double) Math.max(totalCount, 1) / 1024.0,
                totalCount,
                totalBytes / 1024.0 / 1024.0
        );
    }
}
