import { execSync } from 'child_process';
import { existsSync, mkdirSync, rmSync, cpSync, symlinkSync } from 'fs';
import { join } from 'path';

const ROOT = process.cwd();
const APP_PATH = join(ROOT, 'src-tauri/target/release/bundle/macos/Pomotroid Shield.app');
const DMG_DIR = join(ROOT, 'src-tauri/target/release/bundle/dmg');
const DMG_OUT = join(DMG_DIR, 'Pomotroid Shield_1.7.1_aarch64.dmg');
const STAGING = '/tmp/pomotroid_dmg_staging';

console.log('Packaging Pomotroid Shield DMG...');

if (!existsSync(APP_PATH)) {
  console.log('App bundle not found, building Tauri app first...');
  execSync('npm run tauri build -- --bundles app', { stdio: 'inherit' });
}

rmSync(STAGING, { recursive: true, force: true });
mkdirSync(STAGING, { recursive: true });
mkdirSync(DMG_DIR, { recursive: true });

console.log('Staging application and /Applications symlink...');
cpSync(APP_PATH, join(STAGING, 'Pomotroid Shield.app'), { recursive: true });
symlinkSync('/Applications', join(STAGING, 'Applications'));

console.log('Generating compressed UDZO DMG via hdiutil...');
execSync(`hdiutil create -volname "Pomotroid Shield" -srcfolder "${STAGING}" -ov -format UDZO "${DMG_OUT}"`, {
  stdio: 'inherit',
});

rmSync(STAGING, { recursive: true, force: true });
console.log(`\nSuccessfully created DMG:\n${DMG_OUT}`);
