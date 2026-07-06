import { createReadStream, statSync } from "node:fs";
import { createServer } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const port = Number(process.env.PORT ?? 1420);

const contentTypes = {
  ".css": "text/css; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".jpg": "image/jpeg",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".png": "image/png",
};

createServer((request, response) => {
  const url = new URL(request.url ?? "/", `http://127.0.0.1:${port}`);
  const pathname = decodeURIComponent(url.pathname === "/" ? "/index.html" : url.pathname);
  const target = path.normalize(path.join(root, pathname));

  if (!target.startsWith(root)) {
    response.writeHead(403);
    response.end("Forbidden");
    return;
  }

  try {
    const stats = statSync(target);
    if (!stats.isFile()) {
      response.writeHead(404);
      response.end("Not found");
      return;
    }

    response.writeHead(200, {
      "content-type": contentTypes[path.extname(target)] ?? "application/octet-stream",
    });
    createReadStream(target).pipe(response);
  } catch {
    response.writeHead(404);
    response.end("Not found");
  }
}).listen(port, "127.0.0.1", () => {
  console.log(`PiP Kanpe Tool Desktop dev server: http://127.0.0.1:${port}`);
});

