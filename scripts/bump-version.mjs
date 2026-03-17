#!/usr/bin/env node

import { readFileSync, writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");

const version = (process.argv[2] || "").replace(/^v/, "");

if (!/^\d+\.\d+\.\d+/.test(version)) {
  console.error("Usage: node scripts/bump-version.mjs <version>");
  console.error("Example: node scripts/bump-version.mjs 1.2.3");
  process.exit(1);
}

function updateJson(filePath, field = "version") {
  const abs = resolve(root, filePath);
  const data = JSON.parse(readFileSync(abs, "utf-8"));
  data[field] = version;
  writeFileSync(abs, JSON.stringify(data, null, 2) + "\n");
  console.log(`  ✓ ${filePath} → ${version}`);
}

function updateCargoToml(filePath) {
  const abs = resolve(root, filePath);
  const content = readFileSync(abs, "utf-8");
  const updated = content.replace(/^(version\s*=\s*)"[^"]*"/m, `$1"${version}"`);
  writeFileSync(abs, updated);
  console.log(`  ✓ ${filePath} → ${version}`);
}

console.log(`Bumping version to ${version}:\n`);

updateJson("package.json");
updateJson("apps/pomodoro/package.json");
updateJson("apps/pomodoro/src-tauri/tauri.conf.json");
updateCargoToml("apps/pomodoro/src-tauri/Cargo.toml");

console.log("\nDone.");
