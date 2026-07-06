import { cp, mkdir, rm } from "node:fs/promises";
import path from "node:path";

const root = process.cwd();
const dist = path.join(root, "dist");

const entries = [
  "index.html",
  "app.js",
  "core.js",
  "styles.css",
  "favicon.jpg",
  path.join("assets", "maker"),
];

await rm(dist, { recursive: true, force: true });
await mkdir(dist, { recursive: true });

for (const entry of entries) {
  await cp(path.join(root, entry), path.join(dist, entry), {
    recursive: true,
    force: true,
  });
}

