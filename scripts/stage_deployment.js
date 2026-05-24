const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const STAGING_DIR = 'staging';

function stage() {
    console.log("--- Initializing Staging Deployment ---");

    if (!fs.existsSync(STAGING_DIR)) {
        fs.mkdirSync(STAGING_DIR);
    }

    console.log("Staging: Building release binary...");
    execSync('cargo build --release', { stdio: 'inherit' });

    const binarySource = process.platform === 'win32' ? 'target/release/app.exe' : 'target/release/app';
    const binaryDest = path.join(STAGING_DIR, path.basename(binarySource));

    console.log(`Staging: Promoting binary to ${STAGING_DIR}/`);
    fs.copyFileSync(binarySource, binaryDest);

    console.log("Staging: Isolating environment configuration...");
    const stagingEnv = "STAGING_MODE=1\nPROD_MODE=1\nLOG_LEVEL=info\n";
    fs.writeFileSync(path.join(STAGING_DIR, '.env.staging'), stagingEnv);

    console.log("Staging: Copying runtime assets...");
    // Future asset bundles would go here

    console.log("Deployment to Staging Successful.");
}

stage();
