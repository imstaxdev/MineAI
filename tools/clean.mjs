import { rm } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const targets = [
  "target",
  "logs",
  join("apps", "mineia-launcher", "dist"),
  join("apps", "mineia-launcher", ".vite"),
];

for (const target of targets) {
  const absolute = resolve(root, target);
  if (!absolute.startsWith(root)) {
    throw new Error(`Refusing to clean outside repository: ${absolute}`);
  }
  await rm(absolute, { force: true, recursive: true });
  console.log(`removed ${target}`);
}
