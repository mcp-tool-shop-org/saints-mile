#!/usr/bin/env node
/**
 * Generate placeholder PNG assets for MSIX packaging.
 * Creates minimal valid PNGs with a dark background and amber accent.
 * Replace these with real logos before Store submission.
 */
import { writeFileSync, mkdirSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ASSETS_DIR = join(__dirname, "Assets");

// MSIX required logo sizes
const LOGOS = [
  { name: "StoreLogo.png", w: 50, h: 50 },
  { name: "Square44x44Logo.png", w: 44, h: 44 },
  { name: "Square71x71Logo.png", w: 71, h: 71 },
  { name: "Square150x150Logo.png", w: 150, h: 150 },
  { name: "Square310x310Logo.png", w: 310, h: 310 },
  { name: "Wide310x150Logo.png", w: 310, h: 150 },
  // Scaled variants for high-DPI
  { name: "Square44x44Logo.targetsize-24_altform-unplated.png", w: 24, h: 24 },
  { name: "Square44x44Logo.targetsize-32_altform-unplated.png", w: 32, h: 32 },
  { name: "Square44x44Logo.targetsize-48_altform-unplated.png", w: 48, h: 48 },
  { name: "Square44x44Logo.targetsize-256_altform-unplated.png", w: 256, h: 256 },
];

// Background: warm dark brown (#1c1917)
const BG_R = 0x1c, BG_G = 0x19, BG_B = 0x17;
// Accent: amber (#d97706)
const FG_R = 0xd9, FG_G = 0x77, FG_B = 0x06;

function createPng(w, h) {
  const pixels = Buffer.alloc(h * (1 + w * 3));

  for (let y = 0; y < h; y++) {
    const rowOff = y * (1 + w * 3);
    pixels[rowOff] = 0; // no filter
    for (let x = 0; x < w; x++) {
      const px = rowOff + 1 + x * 3;
      const cx = w / 2, cy = h / 2;
      const r = Math.min(w, h) * 0.3;
      const dist = Math.sqrt((x - cx) ** 2 + (y - cy) ** 2);
      if (dist < r) {
        pixels[px] = FG_R;
        pixels[px + 1] = FG_G;
        pixels[px + 2] = FG_B;
      } else {
        pixels[px] = BG_R;
        pixels[px + 1] = BG_G;
        pixels[px + 2] = BG_B;
      }
    }
  }

  const deflated = deflateStored(pixels);
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0);
  ihdr.writeUInt32BE(h, 4);
  ihdr[8] = 8;  // bit depth
  ihdr[9] = 2;  // color type: RGB
  ihdr[10] = 0; // compression
  ihdr[11] = 0; // filter
  ihdr[12] = 0; // interlace

  return Buffer.concat([
    sig,
    makeChunk("IHDR", ihdr),
    makeChunk("IDAT", deflated),
    makeChunk("IEND", Buffer.alloc(0)),
  ]);
}

function makeChunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeB = Buffer.from(type, "ascii");
  const crcData = Buffer.concat([typeB, data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(crcData), 0);
  return Buffer.concat([len, typeB, data, crc]);
}

function deflateStored(data) {
  const MAX_BLOCK = 65535;
  const blocks = [];
  let off = 0;
  while (off < data.length) {
    const remaining = data.length - off;
    const blockSize = Math.min(remaining, MAX_BLOCK);
    const isLast = off + blockSize >= data.length;
    const header = Buffer.alloc(5);
    header[0] = isLast ? 0x01 : 0x00;
    header.writeUInt16LE(blockSize, 1);
    header.writeUInt16LE(blockSize ^ 0xffff, 3);
    blocks.push(header);
    blocks.push(data.subarray(off, off + blockSize));
    off += blockSize;
  }

  const zlibHeader = Buffer.from([0x78, 0x01]);
  const adler = adler32(data);
  const adlerBuf = Buffer.alloc(4);
  adlerBuf.writeUInt32BE(adler, 0);

  return Buffer.concat([zlibHeader, ...blocks, adlerBuf]);
}

function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc ^= buf[i];
    for (let j = 0; j < 8; j++) {
      crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function adler32(buf) {
  let a = 1, b = 0;
  for (let i = 0; i < buf.length; i++) {
    a = (a + buf[i]) % 65521;
    b = (b + a) % 65521;
  }
  return ((b << 16) | a) >>> 0;
}

// Generate
mkdirSync(ASSETS_DIR, { recursive: true });

for (const logo of LOGOS) {
  const png = createPng(logo.w, logo.h);
  const path = join(ASSETS_DIR, logo.name);
  writeFileSync(path, png);
  console.log(`  ${logo.name} (${logo.w}x${logo.h})`);
}

console.log(`\nGenerated ${LOGOS.length} placeholder assets in ${ASSETS_DIR}`);
console.log("Replace with real logos before Store submission.");
