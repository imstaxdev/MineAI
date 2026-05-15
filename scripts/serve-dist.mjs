import { createReadStream, existsSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { dirname, extname, join, normalize, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = normalize(join(dirname(fileURLToPath(import.meta.url)), "..", "apps", "mineia-launcher", "dist"));
const port = Number(process.env.PORT ?? 1420);

const types = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".ico": "image/x-icon",
};

createServer((request, response) => {
  const url = new URL(request.url ?? "/", `http://${request.headers.host}`);
  const safePath = normalize(join(root, decodeURIComponent(url.pathname)));
  const relation = relative(root, safePath);
  const insideRoot = relation === "" || (!relation.startsWith("..") && !relation.includes(":"));
  const filePath = insideRoot && existsSync(safePath) && statSync(safePath).isFile()
    ? safePath
    : join(root, "index.html");

  response.writeHead(200, {
    "Content-Type": types[extname(filePath)] ?? "application/octet-stream",
  });
  createReadStream(filePath).pipe(response);
}).listen(port, "127.0.0.1", () => {
  console.log(`MineIA preview: http://127.0.0.1:${port}`);
});
