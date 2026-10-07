import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { mkdir, readFile } from 'node:fs/promises';

const en = JSON.parse(await readFile(new URL('../src/i18n/en.json', import.meta.url)));
const fa = JSON.parse(await readFile(new URL('../src/i18n/fa.json', import.meta.url)));
assert.deepEqual(Object.keys(en).sort(), Object.keys(fa).sort(), 'Localization keys must match');
await mkdir('test-results', { recursive: true });
const desktop = Boolean(process.env.PIMXSWAP_CDP);
const browser = desktop
  ? await chromium.connectOverCDP(process.env.PIMXSWAP_CDP)
  : await chromium.launch({ executablePath: process.env.CHROME_PATH, headless: true, args: ['--no-proxy-server'] });
const page = desktop ? browser.contexts()[0].pages()[0] : await browser.newPage({ viewport: { width: 1060, height: 790 } });
const pageErrors = [];
page.on('pageerror', e => pageErrors.push(e.message));
let original;
const invoke = (cmd, args) => page.evaluate(({ cmd, args }) => window.__TAURI_INTERNALS__.invoke(cmd, args), { cmd, args });
try {
  page.setDefaultTimeout(60000);
  console.log('Browser ready; loading workspace…');
  if (!desktop) await page.goto('http://127.0.0.1:1420', { waitUntil: 'domcontentloaded', timeout: 60000 });
  await page.locator('.hero').waitFor();
  console.log('Workspace loaded; checking editor…');
  if (desktop) {
    original = (await invoke('bootstrap')).config;
    if (await page.getByRole('dialog').count()) {
      await page.getByRole('dialog').getByRole('button', { name: en.next, exact: true }).click();
      await page.getByRole('dialog').getByRole('button', { name: en.next, exact: true }).click();
      await page.getByRole('button', { name: en.getStarted, exact: true }).click();
      await page.getByRole('dialog').waitFor({ state: 'hidden' });
    }
  } else {
    await page.getByRole('alert').waitFor();
    assert.match(await page.getByRole('alert').innerText(), /desktop app/);
  }
  if (await page.getByRole('button', { name: en.dismiss, exact: true }).count()) await page.getByRole('button', { name: en.dismiss, exact: true }).click();
  await page.locator('#input-text').fill('sghl');
  assert.match(await page.locator('.input-card .editor-footer').innerText(), /4\s+characters/);
  await page.getByRole('button', { name: en.undo, exact: true }).click();
  assert.equal(await page.locator('#input-text').inputValue(), '');
  await page.getByRole('button', { name: en.redo, exact: true }).click();
  assert.equal(await page.locator('#input-text').inputValue(), 'sghl');
  if (desktop) {
    const installed = (await invoke('bootstrap')).layouts;
    const from = installed.find(l => l.language === 'en'), to = installed.find(l => l.language === 'fa');
    assert.ok(from && to, 'Native conversion smoke test needs loaded English and Persian layouts');
    await page.getByRole('button', { name: en.direct, exact: true }).click();
    await page.getByRole('button', { name: en.source, exact: true }).click();
    await page.getByRole('option').filter({ hasText: from.name }).first().click();
    await page.getByRole('button', { name: en.target, exact: true }).click();
    await page.getByRole('option').filter({ hasText: to.name }).first().click();
    await page.locator('#input-text').press('Control+Enter');
    await page.waitForFunction(() => document.querySelector('#output-text').value === 'سلام');
    await page.getByRole('button', { name: en.copy, exact: true }).click();
    assert.equal(await invoke('read_clipboard'), 'سلام');
    await page.getByRole('button', { name: en.swap, exact: true }).click();
    assert.equal(await page.locator('#input-text').inputValue(), 'سلام');
    await page.locator('#input-text').press('Control+Enter');
    await page.waitForFunction(() => document.querySelector('#output-text').value === 'sghl');
  }
  await page.screenshot({ path: 'test-results/workspace-en.png' });
  await page.getByRole('button', { name: en.appLanguage, exact: true }).click();
  await page.waitForFunction(() => document.documentElement.dir === 'rtl');
  assert.equal(await page.locator('html').getAttribute('lang'), 'fa');
  await page.screenshot({ path: 'test-results/workspace-fa.png' });
  await page.getByRole('button', { name: fa.settings, exact: true }).click();
  await page.getByLabel(fa.reduceMotion, { exact: true }).check();
  assert.ok(await page.getByRole('button', { name: fa.save, exact: true }).isEnabled());
  await page.getByRole('button', { name: fa.discard, exact: true }).click();
  if (!desktop) {
    await page.setViewportSize({ width: 600, height: 500 });
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false, 'Settings fit minimum window width');
    await page.getByRole('button', { name: fa.converter, exact: true }).click();
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false, 'Editor fits minimum window width');
    await page.screenshot({ path: 'test-results/workspace-600-rtl.png' });
    await page.getByRole('button', { name: fa.light, exact: true }).click();
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'light');
    await page.screenshot({ path: 'test-results/workspace-light.png' });
  }
  assert.deepEqual(pageErrors, [], 'No runtime JavaScript errors');
  console.log(`${desktop ? 'Native Tauri' : 'Browser'} UI checks passed; screenshots in test-results/.`);
} finally {
  if (original) await invoke('save_settings', { config: original });
  await browser.close();
}
