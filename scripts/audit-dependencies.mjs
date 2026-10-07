import fs from 'node:fs';
const list = JSON.parse(fs.readFileSync('docs/dependency-licenses.json', 'utf8')).filter(x => x.ecosystem === 'Cargo');
try {
  const response = await fetch('https://api.osv.dev/v1/querybatch', {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ queries: list.map(x => ({ package: { name: x.name, ecosystem: 'crates.io' }, version: x.version })) }),
    signal: AbortSignal.timeout(20000),
  });
  if (!response.ok) throw new Error(`OSV returned ${response.status}`);
  const data = await response.json();
  if (data.results?.length !== list.length) throw new Error('Incomplete OSV response');
  const findings = data.results.flatMap((x, i) => (x.vulns ?? []).map(v => ({ package: list[i].name, version: list[i].version, ...v })));
  fs.writeFileSync('test-results/cargo-osv-release.json', JSON.stringify({ checked: new Date().toISOString(), packages: list.length, findings }, null, 2));
  console.log(`OSV: ${list.length} locked Rust dependencies, ${findings.length} known advisories.`);
  if (findings.length) process.exitCode = 1;
} catch (error) {
  fs.writeFileSync('test-results/cargo-osv-release.json', JSON.stringify({ status: 'unavailable', error: String(error) }, null, 2));
  console.error(String(error)); process.exitCode = 1;
}
