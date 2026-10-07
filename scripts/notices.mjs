import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '..');
const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--locked', '--offline', '--format-version', '1', '--filter-platform', 'x86_64-pc-windows-msvc', '--manifest-path', 'src-tauri/Cargo.toml'], { cwd: root, maxBuffer: 32 * 1024 * 1024 }));
const included = new Set(metadata.resolve.nodes.map(n => n.id));
const entries = metadata.packages.filter(p => p.source && included.has(p.id)).map(p => ({ ecosystem: 'Cargo', name: p.name, version: p.version, license: p.license, repository: p.repository, source: `https://static.crates.io/crates/${p.name}/${p.name}-${p.version}.crate`, directory: path.dirname(p.manifest_path) }));
const lock = JSON.parse(fs.readFileSync(path.join(root, 'package-lock.json'), 'utf8'));
for (const [location, info] of Object.entries(lock.packages)) {
  if (!location || info.dev) continue;
  const directory = path.join(root, location);
  const pkg = JSON.parse(fs.readFileSync(path.join(directory, 'package.json'), 'utf8'));
  entries.push({ ecosystem: 'npm', name: pkg.name, version: pkg.version, license: pkg.license, repository: typeof pkg.repository === 'string' ? pkg.repository : pkg.repository?.url, source: info.resolved, directory });
}
const notices = ['PIMXSWAP third-party notices', 'Generated from locked Windows Cargo dependencies (including build dependencies) and production npm dependencies.', 'License identifiers are package metadata. Full available license/notice files follow. This inventory is not a legal compatibility assessment.', ''];
const inventory = [];
for (const item of entries.sort((a, b) => `${a.ecosystem}/${a.name}`.localeCompare(`${b.ecosystem}/${b.name}`))) {
  const files = [];
  for (const entry of fs.readdirSync(item.directory, { withFileTypes: true })) {
    if (!/^(licen[sc]e|copying|notice|copyright)([._-]|$)/i.test(entry.name)) continue;
    const file = path.join(item.directory, entry.name);
    if (entry.isFile()) files.push(file);
    else if (entry.isDirectory()) for (const sub of fs.readdirSync(file, { withFileTypes: true })) if (sub.isFile()) files.push(path.join(file, sub.name));
  }
  const supplemental = path.join(root, 'docs', 'dependency-license-texts', `${item.name}-${item.version}`);
  if (fs.existsSync(supplemental)) for (const name of fs.readdirSync(supplemental)) if (/^(LICENSE|COPYING|NOTICE)/i.test(name)) files.push(path.join(supplemental, name));
  inventory.push({ ecosystem: item.ecosystem, name: item.name, version: item.version, license: item.license, repository: item.repository, source: item.source, licenseFiles: files.map(f => f.startsWith(item.directory) ? path.relative(item.directory, f).replaceAll('\\', '/') : path.relative(root, f).replaceAll('\\', '/')) });
  notices.push('='.repeat(72), `${item.ecosystem}: ${item.name} ${item.version}`, `Declared license: ${item.license ?? 'UNSPECIFIED'}`, `Unmodified dependency source: ${item.source}`, `Repository: ${item.repository ?? 'See source package'}`);
  if (!files.length) notices.push('No standalone license file was provided by this package; review its published source before public distribution.');
  for (const file of files) notices.push(`\n--- ${path.basename(file)} ---\n`, fs.readFileSync(file, 'utf8'));
}
fs.writeFileSync(path.join(root, 'docs/THIRD-PARTY-NOTICES.txt'), notices.join('\n'));
fs.writeFileSync(path.join(root, 'docs/dependency-licenses.json'), JSON.stringify(inventory, null, 2) + '\n');
console.log(`Collected ${entries.length} dependencies; ${inventory.filter(p => !p.licenseFiles.length).length} have no standalone license file.`);
