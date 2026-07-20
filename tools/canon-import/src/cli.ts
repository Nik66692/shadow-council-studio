import { createManifest, writeManifest } from './manifest';
const [, , command, ...args] = process.argv;
if (command !== 'manifest') throw new Error('Usage: canon-import manifest [--dry-run]');
const root = process.cwd();
const manifest = createManifest(root, new Date(0).toISOString());
if (args.includes('--dry-run')) console.log(JSON.stringify(manifest, null, 2));
else console.log(`Wrote ${writeManifest(root, manifest)}`);
