#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export function nextVersion(current, kind) {
  const match = /^(\d+)\.(\d+)\.(\d+)(.*)$/.exec(current);
  if (!match) throw new Error(`Cannot parse version: ${current}`);

  const major = Number(match[1]);
  const minor = Number(match[2]);
  const patch = Number(match[3]);

  switch (kind) {
    case "patch":
      return `${major}.${minor}.${patch + 1}`;
    case "minor":
      return `${major}.${minor + 1}.0`;
    case "major":
      return `${major + 1}.0.0`;
    default:
      throw new Error(`Invalid bump kind: ${kind}. Expected major|minor|patch`);
  }
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

function updateCargoToml(path, version) {
  let inPackage = false;
  const updated = readFileSync(path, "utf8")
    .split("\n")
    .map((line) => {
      if (/^\[\w/.test(line.trim())) inPackage = line.trim() === "[package]";
      if (inPackage && /^version\s*=/.test(line))
        return `version = "${version}"`;
      return line;
    })
    .join("\n");
  writeFileSync(path, updated);
}

function updateCargoLock(path, packageName, version) {
  try {
    const lock = readFileSync(path, "utf8");
    let inTarget = false;
    let replaced = false;
    const updated = lock
      .split("\n")
      .map((line) => {
        if (line.trim() === "[[package]]") {
          inTarget = false;
          return line;
        }
        if (/^name\s*=/.test(line.trim())) {
          inTarget = line.trim() === `name = "${packageName}"`;
          return line;
        }
        if (inTarget && /^version\s*=/.test(line.trim())) {
          replaced = true;
          inTarget = false;
          return `version = "${version}"`;
        }
        return line;
      })
      .join("\n");
    if (replaced) writeFileSync(path, updated);
  } catch (err) {
    if (err.code !== "ENOENT") throw err;
  }
}

function main() {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const repoRoot = join(root, "..", "..");
  const kind = process.argv[2] ?? "patch";

  const tauriConfPath = join(root, "src-tauri", "tauri.conf.json");
  const appPackagePath = join(root, "package.json");
  const rootPackagePath = join(repoRoot, "package.json");
  const cargoTomlPath = join(root, "src-tauri", "Cargo.toml");
  const cargoLockPath = join(root, "src-tauri", "Cargo.lock");

  const tauriConf = readJson(tauriConfPath);
  const current = tauriConf.version;
  const next = nextVersion(current, kind);

  tauriConf.version = next;
  writeJson(tauriConfPath, tauriConf);

  for (const path of [appPackagePath, rootPackagePath]) {
    const pkg = readJson(path);
    pkg.version = next;
    writeJson(path, pkg);
  }

  updateCargoToml(cargoTomlPath, next);
  updateCargoLock(cargoLockPath, "clarus", next);

  process.stdout.write(next);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
