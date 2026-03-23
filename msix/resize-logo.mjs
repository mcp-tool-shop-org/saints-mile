#!/usr/bin/env node
/**
 * Generate real MSIX logo assets from the brand logo.
 * Center-crops to square, then resizes to all required MSIX dimensions.
 * Run: node msix/resize-logo.mjs
 * Requires: npm install sharp (dev dependency, not committed)
 */
import sharp from "sharp";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { mkdirSync } from "fs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ASSETS_DIR = join(__dirname, "Assets");
const BRAND_LOGO = join(__dirname, "..", "..", "brand", "logos", "saints-mile", "readme.png");

mkdirSync(ASSETS_DIR, { recursive: true });

const SQUARE_LOGOS = [
  { name: "StoreLogo.png", size: 50 },
  { name: "Square44x44Logo.png", size: 44 },
  { name: "Square71x71Logo.png", size: 71 },
  { name: "Square150x150Logo.png", size: 150 },
  { name: "Square310x310Logo.png", size: 310 },
  { name: "Square44x44Logo.targetsize-24_altform-unplated.png", size: 24 },
  { name: "Square44x44Logo.targetsize-32_altform-unplated.png", size: 32 },
  { name: "Square44x44Logo.targetsize-48_altform-unplated.png", size: 48 },
  { name: "Square44x44Logo.targetsize-256_altform-unplated.png", size: 256 },
];

const WIDE_LOGOS = [
  { name: "Wide310x150Logo.png", w: 310, h: 150 },
];

// Source is 1536x1024 — crop center 1024x1024 square
const squareBase = sharp(BRAND_LOGO).extract({ left: 256, top: 0, width: 1024, height: 1024 });

for (const logo of SQUARE_LOGOS) {
  const out = join(ASSETS_DIR, logo.name);
  await squareBase.clone().resize(logo.size, logo.size).toFile(out);
  console.log(`  ${logo.name} (${logo.size}x${logo.size})`);
}

for (const logo of WIDE_LOGOS) {
  const out = join(ASSETS_DIR, logo.name);
  await sharp(BRAND_LOGO).resize(logo.w, logo.h, { fit: "cover" }).toFile(out);
  console.log(`  ${logo.name} (${logo.w}x${logo.h})`);
}

console.log(`\nGenerated ${SQUARE_LOGOS.length + WIDE_LOGOS.length} real logo assets in ${ASSETS_DIR}`);
