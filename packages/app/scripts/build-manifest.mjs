#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";

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
const notes = process.env.NOTES ?? `Clarus v${version}`;
const mode = (process.env.MANIFEST_MODE ?? "ci").toLowerCase();

const platformInputs = [
  ["darwin-aarch64", "DARWIN_AARCH64_TARBALL", "DARWIN_AARCH64_SIG_PATH"],
  ["darwin-x86_64", "DARWIN_X86_64_TARBALL", "DARWIN_X86_64_SIG_PATH"],
  ["linux-x86_64", "LINUX_X86_64_TARBALL", "LINUX_X86_64_SIG_PATH"],
  ["windows-x86_64", "WINDOWS_X86_64_TARBALL", "WINDOWS_X86_64_SIG_PATH"],
];

const platforms = {};
for (const [key, artifactEnv, sigEnv] of platformInputs) {
  const artifact = process.env[artifactEnv];
  const sigPath = process.env[sigEnv];
  if (!artifact && !sigPath) continue;
  if (!artifact || !sigPath) {
    console.error(`Set both ${artifactEnv} and ${sigEnv}, or neither.`);
    process.exit(1);
  }
  platforms[key] = {
    signature: readFileSync(sigPath, "utf8").trim(),
    url: `${baseUrl}/${artifact}`,
  };
}

if (mode === "ci" && Object.keys(platforms).length !== platformInputs.length) {
  const missing = platformInputs
    .filter(([key]) => !platforms[key])
    .map(([key]) => key);
  console.error(
    `CI mode requires all updater platforms. Missing: ${missing.join(", ")}`,
  );
  process.exit(1);
}

writeFileSync(
  "latest.json",
  `${JSON.stringify({ version, notes, pub_date: pubDate, platforms }, null, 2)}\n`,
);
