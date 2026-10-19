import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { chromium } from "playwright";

const root = resolve(import.meta.dirname, "..");
const outDir = resolve(root, "landing/assets/screenshots");
const port = 4322;
const baseUrl = `http://127.0.0.1:${port}/capture.html`;

const shots = [
  { id: "shot-dashboard", file: "dashboard.png" },
  { id: "shot-optimize", file: "optimize.png" },
  { id: "shot-benchmark-live", file: "benchmark-live.png" },
  { id: "shot-benchmark", file: "benchmark.png" },
  { id: "shot-performance", file: "performance.png" },
  { id: "shot-diagnostics", file: "diagnostics.png" },
];

async function waitForServer(url, timeoutMs = 90000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch {
      // retry
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`Server not ready: ${url}`);
}

function sha256(buf) {
  return createHash("sha256").update(buf).digest("hex");
}

const preview = spawn("npx", ["vite", "preview", "--host", "127.0.0.1", "--port", String(port)], {
  cwd: root,
  stdio: "inherit",
});

try {
  await new Promise((r) => setTimeout(r, 2000));
  await waitForServer(baseUrl);

  await mkdir(outDir, { recursive: true });
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1200, height: 800 } });
  await page.goto(baseUrl, { waitUntil: "networkidle" });
  await page.waitForTimeout(800);

  const hashes = {};
  for (const shot of shots) {
    const el = page.locator(`#${shot.id}`);
    await el.waitFor({ state: "visible" });
    const path = resolve(outDir, shot.file);
    await el.screenshot({ path });
    const buf = await readFile(path);
    hashes[shot.file] = sha256(buf);
    console.log(`Wrote ${shot.file} sha256=${hashes[shot.file].slice(0, 16)}…`);
  }

  await browser.close();

  console.log("\nScreenshot SHA256:");
  for (const [f, h] of Object.entries(hashes)) {
    console.log(`  ${f}: ${h}`);
  }
} finally {
  preview.kill("SIGTERM");
}
