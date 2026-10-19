#!/usr/bin/env node
/**
 * Push current git tree to GitHub using a PAT in GH_TOKEN / GITHUB_TOKEN.
 * Usage: GH_TOKEN=ghp_... node scripts/sync-github.mjs
 */
import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";

const token = process.env.GH_TOKEN || process.env.GITHUB_TOKEN;
if (!token) {
  console.error("Set GH_TOKEN or GITHUB_TOKEN with repo contents write access.");
  process.exit(1);
}

const owner = "AltinDemaj";
const repo = "boostx";
const branch = "main";

const files = execSync("git ls-files", { encoding: "utf8" })
  .trim()
  .split("\n")
  .filter((p) => p && !p.includes("BoostX-Setup.exe"));

async function gh(path, body, method = "GET") {
  const res = await fetch(`https://api.github.com${path}`, {
    method,
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
      ...(body ? { "Content-Type": "application/json" } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${method} ${path} ${res.status}: ${text}`);
  }
  return res.json();
}

const ref = await gh(`/repos/${owner}/${repo}/git/ref/heads/${branch}`);
const baseSha = ref.object.sha;
const baseCommit = await gh(`/repos/${owner}/${repo}/git/commits/${baseSha}`);
const baseTreeSha = baseCommit.tree.sha;

const tree = [];
for (const path of files) {
  const buf = readFileSync(path);
  const blob = await gh(`/repos/${owner}/${repo}/git/blobs`, {
    content: buf.toString("base64"),
    encoding: "base64",
  });
  tree.push({ path, mode: "100644", type: "blob", sha: blob.sha });
  process.stdout.write(`blob ${path}\n`);
}

const newTree = await gh(`/repos/${owner}/${repo}/git/trees`, {
  base_tree: baseTreeSha,
  tree,
});

const newCommit = await gh(`/repos/${owner}/${repo}/git/commits`, {
  message: `sync from workspace ${new Date().toISOString()}`,
  tree: newTree.sha,
  parents: [baseSha],
});

await gh(`/repos/${owner}/${repo}/git/refs/heads/${branch}`, { sha: newCommit.sha, force: true }, "PATCH");
console.log("Pushed", files.length, "files to", `${owner}/${repo}@${branch}`);
