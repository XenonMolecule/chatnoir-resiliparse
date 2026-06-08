import fs from 'node:fs';
import {WARCParser} from 'warcio';

const BUFFER_SIZE = 4096 << 10;

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

const stream = fs.createReadStream(path, {highWaterMark: BUFFER_SIZE});
const parser = new WARCParser(stream);

for await (const record of parser) {
    let contentLength = 0;
    for await (const chunk of record) {
        contentLength += chunk.length ?? chunk.byteLength ?? 0;
    }

    lastCount += 1;
    lastBytes += contentLength;
    totalCount += 1;
    totalBytes += contentLength;

    const now = process.hrtime.bigint();
    const elapsed = Number(now - lastTimer) / 1e9;
    if (elapsed >= 0.5) {
        console.log(
            `${(lastCount / elapsed).toFixed(0)} records/s, ` +
                `${
                    (lastBytes / elapsed / 1024.0 / 1024.0)
                        .toFixed(1)} MiB/s, ` +
                `${
                    (lastBytes / Math.max(lastCount, 1) / 1024.0)
                        .toFixed(1)} KiB/rec ` +
                `(${totalCount} total, ${
                    (totalBytes / 1024.0 / 1024.0).toFixed(1)} MiB)`,
        );
        lastCount = 0;
        lastBytes = 0;
        lastTimer = now;
    }
}

console.log(
    `Time elapsed: ${
        (Number(process.hrtime.bigint() - start) / 1e9).toFixed(1)}s`,
);
