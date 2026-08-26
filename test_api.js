#!/usr/bin/env node
/**
 * WebSocket API test script for rustsdlretro
 * 
 * Usage:
 *   node test_api.js                          # defaults to ws://localhost:18932
 *   node test_api.js ws://custom-host:port     # custom address
 * 
 * Tests the full API flow: Play → SaveState → LoadState → Step
 */

const { WebSocket } = require('ws');

const url = process.argv[2] || 'ws://localhost:18932';
let passed = 0;
let failed = 0;

function test(name, condition) {
    if (condition) {
        console.log(`  ✅ ${name}`);
        passed++;
    } else {
        console.error(`  ❌ ${name}`);
        failed++;
    }
}

console.log(`\n🧪 rustsdlretro WebSocket API Test`);
console.log(`   Target: ${url}\n`);

const ws = new WebSocket(url);

ws.on('open', () => {
    console.log('✅ Connected to server');
    
    // 1. Test Play command
    sendCommand({ type: 'Play' }, (msg) => {
        if (msg.type === 'Status') {
            test('Play → Status response received', true);
            test(`  Running: ${msg.running}`, msg.running === true);
            test(`  FPS > 0`, msg.fps > 0);
            
            // 2. Test SaveState command
            sendCommand({ type: 'SaveState' }, (msg) => {
                if (msg.type === 'Flash') {
                    test('SaveState → Flash response', true);
                    test(`  Message: "${msg.message}"`, msg.message.includes('Save'));
                    
                    // 3. Test LoadState command
                    sendCommand({ type: 'LoadState' }, (msg) => {
                        if (msg.type === 'Flash') {
                            test('LoadState → Flash response', true);
                            test(`  Message: "${msg.message}"`, msg.message.includes('Load'));
                            
                            // 4. Test Step command
                            sendCommand({ type: 'Step' }, (msg) => {
                                if (msg.type === 'FrameDone') {
                                    test('Step → FrameDone response', true);
                                    finish();
                                } else {
                                    test(`Step → expected FrameDone, got ${msg.type}`, false);
                                    finish();
                                }
                            });
                        } else {
                            test(`LoadState → expected Flash, got ${msg.type}`, false);
                            finish();
                        }
                    });
                } else {
                    test(`SaveState → expected Flash, got ${msg.type}`, false);
                    finish();
                }
            });
        } else {
            test(`Play → expected Status, got ${msg.type}`, false);
            finish();
        }
    });
});

ws.on('error', (err) => {
    console.error(`\n❌ Connection error: ${err.message}`);
    process.exit(1);
});

function sendCommand(cmd, onMessage) {
    const json = JSON.stringify(cmd);
    ws.send(json);
    
    // Listen for the response to this specific command
    const handler = (data) => {
        const msg = JSON.parse(data.toString());
        if (msg.type === cmd.type.replace(/([A-Z])/g, '_$1').toLowerCase()) {
            ws.removeListener('message', handler);
            onMessage(msg);
        } else if (['Status', 'FrameDone', 'Flash'].includes(msg.type)) {
            // These are the expected response types for our commands
            const typeMap = {
                Play: 'Status', SaveState: 'Flash', LoadState: 'Flash', Step: 'FrameDone'
            };
            if (msg.type === typeMap[cmd.type]) {
                ws.removeListener('message', handler);
                onMessage(msg);
            }
        }
    };
    
    ws.on('message', handler);
}

function finish() {
    console.log(`\n📊 Results: ${passed} passed, ${failed} failed`);
    ws.close();
    process.exit(failed > 0 ? 1 : 0);
}

// Timeout after 30 seconds
setTimeout(() => {
    console.error('\n⏱️  Test timed out');
    finish();
}, 30000);
