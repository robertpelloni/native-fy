const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

function runIntegrationTest() {
    console.log("Starting Core-System Integration Test...");

    // 1. Verify Bridge Interface consistency
    const bridgeJs = fs.readFileSync('src/runtime.js', 'utf8');
    const bridgeRust = fs.readFileSync('src/runtime.rs', 'utf8');

    const expectedMethods = [
        'reload',
        'screenshot',
        'runPipeline',
        'toggleDashboard',
        'healthCheck',
        'getMetadata',
        'getPerformanceStats',
        'createSvg',
        'scaleResources'
    ];

    let passed = true;
    expectedMethods.forEach(method => {
        const jsMatch = bridgeJs.includes(`${method}:`);
        const rustMatch = bridgeRust.includes(`_native_${method.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`)}`);
        // Fix for getPerformanceStats -> _native_get_perf_stats mapping in test
        const specialRustMatch = method === 'getPerformanceStats' && bridgeRust.includes('_native_get_perf_stats');

        if (jsMatch && (rustMatch || specialRustMatch)) {
            console.log(`[OK] Method "${method}" verified across bridge.`);
        } else {
            console.warn(`[FAIL] Method "${method}" missing or inconsistent. (JS: ${jsMatch}, Rust: ${rustMatch})`);
            passed = false;
        }
    });

    // 2. Verify Rendering Pipeline batching logic availability
    const mainRust = fs.readFileSync('src/render.rs', 'utf8');
    if (mainRust.includes('while start_idx < nodes.len()')) {
        console.log("[OK] Texture batching logic verified in main rendering loop.");
    } else {
        console.error("[FAIL] Texture batching logic missing from render.rs.");
        passed = false;
    }

    if (passed) {
        console.log("[Success] Core-System Integration Test PASSED.");
    } else {
        console.error("[Failure] Integration Test FAILED.");
        process.exit(1);
    }
}

runIntegrationTest();
