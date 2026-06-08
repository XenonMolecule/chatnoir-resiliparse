package main

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"time"

	"github.com/slyrz/warc"
)

const BufferSize = 4096 << 10

func main() {
	if len(os.Args) != 2 {
		fmt.Printf("Usage: %s WARCFILE\n", os.Args[0])
		return
	}

	path := os.Args[1]
	start := time.Now()
	lastTimer := start
	lastCount := 0
	lastBytes := int64(0)
	totalCount := 0
	totalBytes := int64(0)

	fmt.Printf("Reading WARC file: %s\n", path)
	file, err := os.Open(path)
	if err != nil {
		panic(err)
	}
	defer file.Close()

	bufferedFile := bufio.NewReaderSize(file, BufferSize)
	reader, err := warc.NewReaderMode(bufferedFile, warc.SequentialMode)
	if err != nil {
		panic(err)
	}
	defer reader.Close()

	for {
		record, err := reader.ReadRecord()
		if err != nil {
			break
		}

		contentLength, err := io.Copy(io.Discard, record.Content)
		if err != nil {
			continue
		}

		lastCount++
		lastBytes += contentLength
		totalCount++
		totalBytes += contentLength

		elapsed := time.Since(lastTimer)
		if elapsed >= 500*time.Millisecond {
			fmt.Printf(
				"%.0f records/s, %.1f MiB/s, %.1f KiB/rec (%d total, %.1f MiB)\n",
				float64(lastCount)/elapsed.Seconds(),
				float64(lastBytes)/elapsed.Seconds()/1024.0/1024.0,
				float64(lastBytes)/float64(max(lastCount, 1))/1024.0,
				totalCount,
				float64(totalBytes)/1024.0/1024.0,
			)
			lastCount = 0
			lastBytes = 0
			lastTimer = time.Now()
		}
	}

	fmt.Printf("Time elapsed: %.1fs\n", time.Since(start).Seconds())
}
