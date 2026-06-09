'use strict';

const fs = require('fs');
const zlib = require('zlib');
const {recordIterator} = require('node-warc');

const DEFAULT_BUFFER_SIZE = 1024 << 10;

function bufferSize() {
    const value = Number.parseInt(process.env.BUFFER_SIZE ?? '', 10);
    return Number.isFinite(value) && value > 0 ? value : DEFAULT_BUFFER_SIZE;
}

if (process.argv.length !== 3) {
    console.log(`Usage: ${process.argv[1]} WARCFILE`);
    process.exit(0);
}

const path = process.argv[2];
const start = process.hrtime.bigint();
let lastTimer = start;
let lastCount = 0;
let lastBytes = 0;
let totalCount = 0;
let totalBytes = 0;

console.log(`Reading WARC file: ${path}`);

function printFinal() {
    const totalElapsed = Number(process.hrtime.bigint() - start) / 1e9;
    console.log(
        `Summary: ${totalElapsed.toFixed(1)}s, ` +
        `${(totalCount / totalElapsed).toFixed(0)} records/s, ` +
        `${(totalBytes / totalElapsed / 1024.0 / 1024.0).toFixed(1)} MiB/s, ` +
        `${(totalBytes / Math.max(totalCount, 1) / 1024.0).toFixed(1)} KiB/rec ` +
        `(${totalCount} total, ${(totalBytes / 1024.0 / 1024.0).toFixed(1)} MiB)`,
    );
}

const bufferSizeValue = bufferSize();
let stream = fs.createReadStream(path, {highWaterMark: bufferSizeValue});
if (path.endsWith('.gz')) {
    stream = stream.pipe(zlib.createGunzip({chunkSize: bufferSizeValue}));
}

(async () => {
    for await (const record of recordIterator(stream)) {
        const parsedLength = Number.parseInt(record.warcContentLength, 10);
        const contentLength = Number.isFinite(parsedLength) && parsedLength > 0 ? parsedLength : 0;

        lastCount += 1;
        lastBytes += contentLength;
        totalCount += 1;
        totalBytes += contentLength;

        const now = process.hrtime.bigint();
        const elapsed = Number(now - lastTimer) / 1e9;
        if (elapsed >= 0.5) {
            console.log(
                `${(lastCount / elapsed).toFixed(0)} records/s, ` +
                `${(lastBytes / elapsed / 1024.0 / 1024.0).toFixed(1)} MiB/s, ` +
                `${(lastBytes / Math.max(lastCount, 1) / 1024.0).toFixed(1)} KiB/rec ` +
                `(${totalCount} total, ${(totalBytes / 1024.0 / 1024.0).toFixed(1)} MiB)`,
            );
            lastCount = 0;
            lastBytes = 0;
            lastTimer = now;
        }
    }

    printFinal();
})().catch((err) => {
    if (err && err.code === 'ERR_MULTIPLE_CALLBACK') {
        printFinal();
        return;
    }
    console.error(err);
    process.exit(1);
});
