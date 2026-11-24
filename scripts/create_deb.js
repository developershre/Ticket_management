#!/usr/bin/env node
/**
 * Cross-platform pure Node.js .deb package generator.
 * Creates standard Debian binary packages (.deb) using standard ar + tar.gz formats.
 * Works on Windows, Linux, macOS without needing dpkg-deb or system ar.
 */

const fs = require('fs');
const path = require('path');
const zlib = require('zlib');
const { execSync } = require('child_process');

function createArHeader(filename, size, mtime = Math.floor(Date.now() / 1000)) {
    const buf = Buffer.alloc(60, 0x20); // space-padded
    const name = (filename.endsWith('/') ? filename : filename + '/').padEnd(16, ' ');
    buf.write(name, 0, 16, 'ascii');
    buf.write(String(mtime).padEnd(12, ' '), 16, 12, 'ascii');
    buf.write('0'.padEnd(6, ' '), 28, 6, 'ascii'); // owner
    buf.write('0'.padEnd(6, ' '), 34, 6, 'ascii'); // group
    buf.write('100644'.padEnd(8, ' '), 40, 8, 'ascii'); // mode
    buf.write(String(size).padEnd(10, ' '), 48, 10, 'ascii'); // size
    buf.write('`\n', 58, 2, 'ascii'); // magic trailer
    return buf;
}

function packAr(entries, outputPath) {
    const buffers = [Buffer.from('!<arch>\n', 'ascii')];

    for (const entry of entries) {
        const header = createArHeader(entry.name, entry.data.length, entry.mtime);
        buffers.push(header);
        buffers.push(entry.data);
        if (entry.data.length % 2 !== 0) {
            buffers.push(Buffer.from('\n', 'ascii')); // 2-byte alignment
        }
    }

    const finalBuffer = Buffer.concat(buffers);
    fs.writeFileSync(outputPath, finalBuffer);
}

function createTarGzFromDir(sourceDir, relativePrefix = '.') {
    // If tar command is available, use it for perfect UNIX permissions
    const tmpTar = path.join(sourceDir, '..', `_tmp_${Date.now()}_${Math.random().toString(36).substring(2)}.tar.gz`);
    try {
        const cmd = `tar -czf "${tmpTar.replace(/\\/g, '/')}" -C "${sourceDir.replace(/\\/g, '/')}" ${relativePrefix}`;
        execSync(cmd, { stdio: 'pipe' });
        const buf = fs.readFileSync(tmpTar);
        fs.unlinkSync(tmpTar);
        return buf;
    } catch {
        // Fallback: minimal in-memory tar.gz creator
        return fallbackTarGz(sourceDir);
    }
}

function fallbackTarGz(sourceDir) {
    const chunks = [];
    function walk(dir, relPath) {
        const items = fs.readdirSync(dir);
        for (const item of items) {
            const full = path.join(dir, item);
            const rel = relPath ? `${relPath}/${item}` : item;
            const stat = fs.statSync(full);
            if (stat.isDirectory()) {
                appendTarEntry(chunks, rel + '/', Buffer.alloc(0), 0o755, stat.mtimeMs, '5');
                walk(full, rel);
            } else if (stat.isFile()) {
                const data = fs.readFileSync(full);
                const mode = (item.endsWith('.sh') || rel.includes('bin/')) ? 0o755 : 0o644;
                appendTarEntry(chunks, rel, data, mode, stat.mtimeMs, '0');
            }
        }
    }
    walk(sourceDir, '.');
    chunks.push(Buffer.alloc(1024)); // tar EOF block (2x 512 bytes)
    const tarBuf = Buffer.concat(chunks);
    return zlib.gzipSync(tarBuf);
}

function appendTarEntry(chunks, name, data, mode, mtimeMs, typeflag) {
    const header = Buffer.alloc(512);
    header.write(name, 0, 100, 'utf8');
    header.write(mode.toString(8).padStart(7, '0'), 100, 8, 'ascii'); // mode
    header.write('0000000', 108, 8, 'ascii'); // uid
    header.write('0000000', 116, 8, 'ascii'); // gid
    header.write(data.length.toString(8).padStart(11, '0'), 124, 12, 'ascii'); // size
    const mtimeSec = Math.floor(mtimeMs / 1000);
    header.write(mtimeSec.toString(8).padStart(11, '0'), 136, 12, 'ascii'); // mtime
    header.write('        ', 148, 8, 'ascii'); // checksum space placeholder
    header.write(typeflag, 156, 1, 'ascii'); // typeflag
    header.write('ustar\0', 257, 6, 'ascii'); // ustar magic
    header.write('00', 263, 2, 'ascii');

    // Calculate tar checksum
    let sum = 0;
    for (let i = 0; i < 512; i++) {
        sum += header[i];
    }
    header.write(sum.toString(8).padStart(6, '0') + '\0 ', 148, 8, 'ascii');

    chunks.push(header);
    if (data.length > 0) {
        chunks.push(data);
        const pad = (512 - (data.length % 512)) % 512;
        if (pad > 0) {
            chunks.push(Buffer.alloc(pad));
        }
    }
}

function main() {
    const args = process.argv.slice(2);
    if (args.length < 2) {
        console.error('Usage: node create_deb.js <stage_dir> <output_file.deb>');
        process.exit(1);
    }

    const stageDir = path.resolve(args[0]);
    const outputFile = path.resolve(args[1]);
    const debianDir = path.join(stageDir, 'DEBIAN');

    if (!fs.existsSync(debianDir)) {
        console.error(`Error: DEBIAN metadata directory not found at ${debianDir}`);
        process.exit(1);
    }

    console.log(`[create_deb] Packaging ${stageDir} -> ${outputFile}...`);

    // 1. debian-binary
    const debianBinary = Buffer.from('2.0\n', 'ascii');

    // 2. control.tar.gz (content of DEBIAN/ folder)
    const controlTarGz = createTarGzFromDir(debianDir, '.');

    // 3. data.tar.gz (content of stageDir excluding DEBIAN/)
    // We create a temporary staging folder for data or tar directly
    const tmpDataDir = path.join(stageDir, '..', `_data_${Date.now()}`);
    fs.mkdirSync(tmpDataDir, { recursive: true });
    
    // Copy all folders except DEBIAN into tmpDataDir
    const items = fs.readdirSync(stageDir);
    for (const item of items) {
        if (item === 'DEBIAN') continue;
        const src = path.join(stageDir, item);
        const dst = path.join(tmpDataDir, item);
        fs.cpSync(src, dst, { recursive: true });
    }

    const dataTarGz = createTarGzFromDir(tmpDataDir, '.');
    fs.rmSync(tmpDataDir, { recursive: true, force: true });

    // 4. Assemble standard ar archive
    packAr([
        { name: 'debian-binary', data: debianBinary },
        { name: 'control.tar.gz', data: controlTarGz },
        { name: 'data.tar.gz', data: dataTarGz }
    ], outputFile);

    console.log(`[create_deb] Successfully built: ${outputFile} (${Math.round(fs.statSync(outputFile).size / 1024)} KB)`);
}

main();
