import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '..');
const temporary = await fs.mkdtemp(path.join(root, '.build/management-smoke-'));
const binary = path.resolve(root, process.env.CARGO_TARGET_DIR ?? 'target', 'debug/luma');
const run = (command, args) => {
  const result = spawnSync(command, args, { cwd: root, encoding: 'utf8', timeout: 120000 });
  assert.equal(result.status, 0, result.stderr || result.error?.message);
};
try {
  if (process.env.LUMA_SMOKE_SKIP_BUILD !== '1')
    run('cargo', ['build', '--locked', '--package', 'luma']);
  const preferences = path.join(temporary, 'preferences.json');
  const scenario = async (name, script) => {
    const dump = path.join(temporary, `${name}.json`);
    run(binary, [
      '--preferences',
      preferences,
      '--script',
      script,
      '--frames',
      '100',
      '--dump-state',
      dump,
      '--screenshot',
      path.join(root, '.build', `management-${name}.png`),
    ]);
    return JSON.parse(await fs.readFile(dump, 'utf8'));
  };
  const actions = 'settings,click:300:536,down,enter';
  const listing = await scenario('list', actions);
  assert.deepEqual(listing.subpage, { Management: 'SmartActions' });
  assert.ok(listing.ui_text.includes('Create Smart Action'));
  const creation = `${actions},enter,enter,type:Gaming,back,down,down,enter,down,enter`;
  const editor = await scenario('editor', creation);
  assert.deepEqual(editor.subpage, { Management: 'SmartEditor' });
  assert.ok(editor.ui_text.includes('Name: Gaming'));
  const saved = await scenario('saved', `${creation},${Array(6).fill('down').join(',')},enter`);
  assert.equal(saved.preferences.smart_actions[0].name, 'Gaming');
  assert.deepEqual(saved.preferences.smart_actions[0].steps, [{ kind: 'refresh_inputs' }]);
  const loaded = await scenario('persisted', actions);
  assert.equal(loaded.preferences.smart_actions[0].name, 'Gaming');
  const diagnostics = await scenario('diagnostics', 'home,click:1670:64,down,down,enter');
  assert.deepEqual(diagnostics.subpage, { Management: 'Diagnostics' });
  // The service panel remains remote navigable and exposes actual system observations.
  assert.ok(diagnostics.ui_text.some((text) => text.includes('Unavailable')));
  console.log('Management smoke passed: remote create/name/step/save/reload and diagnostics.');
} finally {
  await fs.rm(temporary, { recursive: true, force: true });
}
