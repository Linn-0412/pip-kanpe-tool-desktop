import { readFile } from "node:fs/promises";

const textFiles = new Map();

async function readText(filePath) {
  if (!textFiles.has(filePath)) {
    textFiles.set(filePath, await readFile(filePath, "utf8"));
  }
  return textFiles.get(filePath);
}

function assertEqual(label, actual, expected) {
  if (actual !== expected) {
    throw new Error(`${label} is ${actual}, expected ${expected}`);
  }
}

function assertIncludes(label, content, expected) {
  if (!content.includes(expected)) {
    throw new Error(`${label} does not include ${expected}`);
  }
}

function matchVersion(label, content, pattern) {
  const match = content.match(pattern);
  if (!match) {
    throw new Error(`${label} version could not be found`);
  }
  return match[1];
}

const packageJson = JSON.parse(await readText("package.json"));
const version = packageJson.version;

if (!/^\d+\.\d+\.\d+$/.test(version)) {
  throw new Error(`package.json version must be semver x.y.z, got ${version}`);
}

const packageLock = JSON.parse(await readText("package-lock.json"));
assertEqual("package-lock.json version", packageLock.version, version);
assertEqual("package-lock root version", packageLock.packages?.[""]?.version, version);

const tauriConf = JSON.parse(await readText("src-tauri/tauri.conf.json"));
assertEqual("src-tauri/tauri.conf.json version", tauriConf.version, version);

const cargoToml = await readText("src-tauri/Cargo.toml");
assertEqual(
  "src-tauri/Cargo.toml version",
  matchVersion("src-tauri/Cargo.toml", cargoToml, /^version\s*=\s*"([^"]+)"/m),
  version,
);

const cargoLock = await readText("src-tauri/Cargo.lock");
assertEqual(
  "src-tauri/Cargo.lock package version",
  matchVersion(
    "src-tauri/Cargo.lock package",
    cargoLock,
    /\[\[package\]\]\r?\nname = "pip-kanpe-tool-desktop"\r?\nversion = "([^"]+)"/,
  ),
  version,
);

const indexHtml = await readText("index.html");
assertIncludes("index.html stylesheet cache query", indexHtml, `styles.css?v=${version}`);
assertIncludes("index.html script cache query", indexHtml, `app.js?v=${version}`);

const pipHtml = await readText("pip.html");
assertIncludes("pip.html stylesheet cache query", pipHtml, `styles.css?v=${version}`);
assertIncludes("pip.html script cache query", pipHtml, `pip-window.js?v=${version}`);

console.log(`Release metadata validated for v${version}.`);
