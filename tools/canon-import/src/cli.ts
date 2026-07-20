import { CanonManifestError, createManifest, writeManifest } from "./manifest";

const [, , command, ...args] = process.argv;

try {
  if (command !== "manifest") {
    throw new CanonManifestError("Usage: canon-import manifest [--dry-run]");
  }

  const manifest = createManifest(process.cwd(), new Date(0).toISOString());
  if (args.includes("--dry-run")) {
    console.log(JSON.stringify(manifest, null, 2));
  } else {
    console.log(`Wrote ${writeManifest(process.cwd(), manifest)}`);
  }
} catch (error) {
  if (error instanceof CanonManifestError) {
    console.error(error.message);
    process.exitCode = 1;
  } else {
    throw error;
  }
}
