// Downstream fixes are versioned here until their upstream issues are resolved.
// Apply only to the published submodule revision recorded by this repository.
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const compiler = resolve(root, "rust-js");
const patches = resolve(root, "patches/rust-js");

function git(cwd: string, args: string[]) {
  return Bun.spawnSync(["git", ...args], {
    cwd,
    stdout: "pipe",
    stderr: "pipe",
    timeout: 10_000,
  });
}

const index = git(root, ["ls-files", "--stage", "--", "rust-js"]);
const head = git(compiler, ["rev-parse", "HEAD"]);
const pinned = index.stdout.toString().trim().split(/\s+/)[1];
if (index.exitCode !== 0 || head.exitCode !== 0 || !pinned || pinned !== head.stdout.toString().trim()) {
  throw new Error("Initialize rust-js at the repository's pinned submodule revision before applying patches: git submodule update --init --recursive");
}

const files = readdirSync(patches).filter(file => file.endsWith(".patch")).sort();
// Later patches may edit earlier patches' context. Prove the whole stack is
// applied by reversing a disposable copy, never the user's checkout.
const probe = mkdtempSync(join(tmpdir(), "vgpu-patches-"));
let alreadyApplied = false;
try {
  const paths = new Set(files.flatMap(file => [...readFileSync(resolve(patches, file), "utf8").matchAll(/^\+\+\+ b\/(.+)$/gm)].map(match => match[1])));
  for (const path of paths) {
    const from = resolve(compiler, path);
    if (existsSync(from)) {
      const to = resolve(probe, path);
      mkdirSync(dirname(to), { recursive: true });
      copyFileSync(from, to);
    }
  }
  if (git(probe, ["init", "--quiet"]).exitCode !== 0) throw new Error("Cannot initialize patch verification");
  alreadyApplied = [...files].reverse().every(file => git(probe, ["apply", "--reverse", resolve(patches, file)]).exitCode === 0);
} finally {
  rmSync(probe, { recursive: true, force: true });
}
if (alreadyApplied) console.log("rust-js patch stack already applied.");
for (const file of alreadyApplied ? [] : files) {
  const patch = resolve(patches, file);
  const check = git(compiler, ["apply", "--check", patch]);
  if (check.exitCode === 0) {
    const applied = git(compiler, ["apply", patch]);
    if (applied.exitCode !== 0) throw new Error(applied.stderr.toString());
    console.log(`Applied rust-js patch: ${file}`);
  } else if (git(compiler, ["apply", "--reverse", "--check", patch]).exitCode === 0) {
    console.log(`rust-js patch already applied: ${file}`);
  } else {
    throw new Error(`rust-js patch ${file} does not apply cleanly; inspect the checkout and patch before proceeding.\n${check.stderr}`);
  }
}
