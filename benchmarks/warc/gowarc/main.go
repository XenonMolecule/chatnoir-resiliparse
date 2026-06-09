package main

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"strconv"
	"time"

	"github.com/internetarchive/gowarc"
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

	if err := os.Setenv("WARCDecompressedBufSize", strconv.Itoa(BufferSize)); err != nil {
		panic(err)
	}

	reader, err := warc.NewReader(bufio.NewReaderSize(file, BufferSize))
	if err != nil {
		panic(err)
	}
	defer reader.Close()

	for {
		record, err := reader.ReadRecord(warc.ReadOptsNoContentOutput)
		if err != nil {
			if err != io.EOF {
				fmt.Fprintf(os.Stderr, "error: %v\n", err)
			}
			break
		}

		contentLength, err := strconv.ParseInt(record.Header.Get("Content-Length"), 10, 64)
		if err != nil || contentLength < 0 {
			contentLength = 0
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

	totalElapsed := time.Since(start).Seconds()
	fmt.Printf(
		"Summary: %.1fs, %.0f records/s, %.1f MiB/s, %.1f KiB/rec (%d total, %.1f MiB)\n",
		totalElapsed,
		float64(totalCount)/totalElapsed,
		float64(totalBytes)/totalElapsed/1024.0/1024.0,
		float64(totalBytes)/float64(max(totalCount, 1))/1024.0,
		totalCount,
		float64(totalBytes)/1024.0/1024.0,
	)
}
