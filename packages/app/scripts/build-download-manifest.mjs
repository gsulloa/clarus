#!/usr/bin/env node
import { statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

function required(name) {
  const value = process.env[name];
  if (!value) {
    console.error(`Missing required env var: ${name}`);
    process.exit(1);
  }
  return value;
}

const version = required("VERSION");
const pubDate = required("PUB_DATE");
const baseUrl = required("PUBLIC_URL_BASE").replace(/\/+$/, "");
const mode = (process.env.MANIFEST_MODE ?? "ci").toLowerCase();
const stagingDir = process.env.STAGING_DIR ?? "staging";

const platformInputs = [
  ["darwin-aarch64", "DARWIN_AARCH64_INSTALLER"],
  ["darwin-x86_64", "DARWIN_X86_64_INSTALLER"],
  ["linux-x86_64", "LINUX_X86_64_INSTALLER"],
  ["windows-x86_64", "WINDOWS_X86_64_INSTALLER"],
];

const forbiddenSuffixes = [
  ".app.tar.gz",
  ".AppImage.tar.gz",
  ".msi.zip",
  ".sig",
];
const installers = {};

for (const [key, envName] of platformInputs) {
  const filename = process.env[envName];
  if (!filename) continue;
  if (forbiddenSuffixes.some((suffix) => filename.endsWith(suffix))) {
    console.error(
      `${filename} is an updater artifact, not an end-user installer.`,
    );
    process.exit(1);
  }
  const size = statSync(join(stagingDir, filename)).size;
  installers[key] = {
    url: `${baseUrl}/${filename}`,
    filename,
    size,
  };
}

if (mode === "ci" && Object.keys(installers).length !== platformInputs.length) {
  const missing = platformInputs
    .filter(([key]) => !installers[key])
    .map(([key]) => key);
  console.error(
    `CI mode requires all installers. Missing: ${missing.join(", ")}`,
  );
  process.exit(1);
}

writeFileSync(
  "download.json",
  `${JSON.stringify({ version, pub_date: pubDate, installers }, null, 2)}\n`,
);
