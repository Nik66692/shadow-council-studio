import { existsSync } from "node:fs";
import path from "node:path";
import { CanonManifestError, createManifest, writeManifest } from "./manifest";

function findRepositoryRoot(startDirectory: string): string {
  let current = path.resolve(startDirectory);

  while (true) {
    if (existsSync(path.join(current, "pnpm-workspace.yaml"))) {
      return current;
    }

    const parent = path.dirname(current);
    if (parent === current) {
      throw new CanonManifestError(
        `Could not locate repository root from ${startDirectory}.`,
      );
    }
    current = parent;
  }
}

const [, , command, ...args] = process.argv;

try {
  if (command !== "manifest") {
    throw new CanonManifestError("Usage: canon-import manifest [--dry-run]");
  }

  const repositoryRoot = findRepositoryRoot(process.cwd());
  const manifest = createManifest(repositoryRoot, new Date(0).toISOString());
  if (args.includes("--dry-run")) {
    console.log(JSON.stringify(manifest, null, 2));
  } else {
    console.log(`Wrote ${writeManifest(repositoryRoot, manifest)}`);
  }
} catch (error) {
  if (error instanceof CanonManifestError) {
    console.error(error.message);
    process.exitCode = 1;
  } else {
    throw error;
  }
}
