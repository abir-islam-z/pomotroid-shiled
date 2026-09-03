import { execSync } from 'child_process';
import { existsSync, mkdirSync, rmSync, cpSync, symlinkSync } from 'fs';
import { join } from 'path';

const ROOT = process.cwd();
const MACOS_DIR = join(ROOT, 'src-tauri/target/release/bundle/macos');
const APP_PATH = join(MACOS_DIR, 'Pomotroid Shield.app');
const DMG_DIR = join(ROOT, 'src-tauri/target/release/bundle/dmg');
const DMG_OUT = join(DMG_DIR, 'Pomotroid Shield_1.7.1_aarch64.dmg');
const STAGING = '/tmp/pomotroid_dmg_staging';

console.log('1. Building Tauri application in release mode...');
execSync('npm run tauri build -- --bundles app', { stdio: 'inherit' });

console.log('2. Staging application and /Applications symlink...');
rmSync(STAGING, { recursive: true, force: true });
mkdirSync(STAGING, { recursive: true });
mkdirSync(DMG_DIR, { recursive: true });

cpSync(APP_PATH, join(STAGING, 'Pomotroid Shield.app'), { recursive: true });
symlinkSync('/Applications', join(STAGING, 'Applications'));

console.log('3. Generating compressed UDZO DMG installer via hdiutil...');
execSync(`hdiutil create -volname "Pomotroid Shield" -srcfolder "${STAGING}" -ov -format UDZO "${DMG_OUT}"`, {
  stdio: 'inherit',
});

// Clean up staging
rmSync(STAGING, { recursive: true, force: true });

// Explicit user request: "just build the dmg don't build .app. Even if .app exists delete it."
console.log('4. Removing all .app bundles as requested (leaving only .dmg)...');
if (existsSync(MACOS_DIR)) {
  rmSync(MACOS_DIR, { recursive: true, force: true });
  console.log(`Deleted: ${MACOS_DIR}`);
}

console.log(`\nBuild complete! Only the DMG installer exists:\n${DMG_OUT}\n`);
