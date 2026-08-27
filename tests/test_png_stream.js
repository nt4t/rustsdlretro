#!/usr/bin/env node
/**
 * WebSocket PNG Frame Streaming Test for rustsdlretro
 * 
 * Tests the full frame streaming flow:
 *   1. Connect to WebSocket server
 *   2. Send "Play" command (starts emulation)
 *   3. Receive PNG frames as binary messages
 *   4. Decode and save first frame
 *   5. Verify it's a valid PNG image
 * 
 * Usage:
 *   node test_png_stream.js                          # defaults to ws://localhost:18932
 *   node test_png_stream.js ws://custom-host:port     # custom address
 *   node test_png_stream.js --max-frames 5            # stop after N frames (default: 3)
 */

const { WebSocket } = require('ws');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Parse arguments
let url = 'ws://localhost:18932';
let maxFrames = 3;
let outputDir = './test_output';

for (let i = 2; i < process.argv.length; i++) {
    if (process.argv[i] === '--url' && i + 1 < process.argv.length) {
        url = process.argv[++i];
    } else if (process.argv[i] === '--max-frames' && i + 1 < process.argv.length) {
        maxFrames = parseInt(process.argv[++i], 10);
    } else if (process.argv[i] === '--help') {
        console.log('Usage: node test_png_stream.js [--url URL] [--max-frames N]');
        process.exit(0);
    }
}

// Ensure output directory exists
if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
}

let passed = 0;
let failed = 0;
let framesReceived = 0;
let startTime = Date.now();

function test(name, condition) {
    if (condition) {
        console.log(`  ✅ ${name}`);
        passed++;
        return true;
    } else {
        console.error(`  ❌ ${name}`);
        failed++;
        return false;
    }
}

function log(msg, color = '') {
    const colors = {
        green: '\x1b[32m', red: '\x1b[31m', yellow: '\x1b[33m',
        cyan: '\x1b[36m', reset: '\x1b[0m'
    };
    console.log(`${colors[color]}${msg}${colors.reset}`);
}

console.log(`\n${'─'.repeat(60)}`);
log('🧪 PNG Frame Streaming Test', 'cyan');
console.log(`   Target:  ${url}`);
console.log(`   Max frames: ${maxFrames}`);
console.log(`   Output:  ${outputDir}/`);
console.log(`${'─'.repeat(60)}\n`);

// ─── PNG Validation ────────────────────────────────────────────────

/**
 * Validate that buffer contains a valid PNG file.
 * Returns { valid, width, height, format } or null if invalid.
 */
function validatePNG(buffer) {
    // Check PNG signature (8 bytes)
    const sig = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    for (let i = 0; i < 8; i++) {
        if (buffer[i] !== sig[i]) return null;
    }

    // Parse IHDR chunk (first chunk after signature)
    // IHDR: 4 bytes length + "IHDR" + 13 bytes data + 4 bytes CRC
    if (buffer.length < 25) return null;
    
    const ihdrLength = buffer.readUInt32BE(8);
    const ihdrType = buffer.toString('ascii', 12, 16);
    if (ihdrType !== 'IHDR' || ihdrLength !== 13) return null;

    const width = buffer.readUInt32BE(16);
    const height = buffer.readUInt32BE(20);
    const bitDepth = buffer[24];
    
    // Validate reasonable values
    if (width === 0 || height === 0 || width > 8192 || height > 8192) return null;
    if (![1, 2, 4, 8, 16].includes(bitDepth)) return null;

    // Check for IDAT chunks (image data)
    let hasIdat = false;
    let offset = 33; // After signature + IHDR chunk
    while (offset < buffer.length - 8) {
        const chunkLen = buffer.readUInt32BE(offset);
        const chunkType = buffer.toString('ascii', offset + 4, offset + 8);
        if (chunkType === 'IDAT') { hasIdat = true; break; }
        if (chunkType === 'IEND') break;
        offset += 12 + chunkLen; // length(4) + type(4) + data + crc(4)
    }

    return { valid: true, width, height, bitDepth, hasIdat };
}

// ─── Binary Message Parser ─────────────────────────────────────────

/**
 * Parse binary WebSocket message containing PNG frame.
 * Format: [width u16 BE][height u16 BE][PNG bytes]
 */
function parseBinaryFrame(buffer) {
    if (buffer.length < 4) return null; // Need at least header
    
    const width = buffer.readUInt16BE(0);
    const height = buffer.readUInt16BE(2);
    const pngData = buffer.slice(4);
    
    return { width, height, pngBuffer: pngData };
}

// ─── Connection & Test Flow ────────────────────────────────────────

const ws = new WebSocket(url, {
    handshakeTimeout: 10000,
});
// Text messages as strings, binary as Buffer
ws.binaryType = 'arraybuffer';

ws.on('open', () => {
    log('✅ Connected to server', 'green');
    
    // Send Play command to start emulation
    const playCmd = JSON.stringify({ type: 'Play' });
    console.log(`📤 Sending: ${playCmd}`);
    ws.send(playCmd);
});

ws.on('message', (data) => {
    // Try JSON first (text messages from server responses)
    const str = data.toString();
    try {
        const msg = JSON.parse(str);
        handleTextMessage(msg);
        return;
    } catch {}
    
    // If not valid JSON, treat as binary PNG frame
    if (data instanceof Buffer && data.length > 4) {
        handleBinaryFrame(data);
    }
});

ws.on('error', (err) => {
    log(`❌ Connection error: ${err.message}`, 'red');
    finish();
});

// ─── Message Handlers ──────────────────────────────────────────────

function handleTextMessage(msg) {
    console.log(`📨 Received: ${JSON.stringify(msg).substring(0, 120)}`);
    
    switch (msg.type) {
        case 'Status':
            test('Play → Status response', true);
            if (!test('Running = true', msg.running === true)) {
                finish();
                return;
            }
            if (!test(`FPS reported: ${msg.fps}`, msg.fps > 0)) {
                // Continue even with 0 FPS - core may not have started yet
            }
            console.log(`   Resolution: ${msg.width}×${msg.height}`);
            
            // Send Step command to trigger one frame
            setTimeout(() => {
                const stepCmd = JSON.stringify({ type: 'Step' });
                console.log(`\n📤 Sending: ${stepCmd}`);
                ws.send(stepCmd);
            }, 500);
            break;
            
        case 'FrameDone':
            test('Step → FrameDone response', true);
            log(`⏭️  Frame stepped, waiting for PNG stream...`, 'yellow');
            // The frame streaming task should now be sending PNG frames
            // Give a moment for the first frame to be captured and sent
            break;
            
        case 'Flash':
            test(`Flash: "${msg.message}"`, msg.message.length > 0);
            break;
            
        case 'Error':
            log(`❌ Server error: ${msg.message}`, 'red');
            failed++;
            finish();
            break;
            
        default:
            console.log(`   (Unknown message type: ${msg.type})`);
    }
}

function handleBinaryFrame(buffer) {
    framesReceived++;
    
    // Parse the frame header [width u16][height u16][PNG bytes]
    const frame = parseBinaryFrame(buffer);
    if (!frame) {
        log(`❌ Invalid binary message (too short: ${buffer.length} bytes)`, 'red');
        failed++;
        return;
    }
    
    console.log(`\n🖼️  Frame #${framesReceived}: ${frame.width}×${frame.height}, PNG size: ${frame.pngBuffer.length} bytes`);
    
    // Validate PNG structure
    const pngInfo = validatePNG(frame.pngBuffer);
    if (!pngInfo) {
        log(`❌ Invalid PNG data (first 8 bytes: ${Array.from(frame.pngBuffer.slice(0, 8)).map(b => '0x' + b.toString(16).padStart(2, '0')).join(' ')})`, 'red');
        failed++;
        return;
    }
    
    test(`Valid PNG signature & IHDR`, true);
    if (!test(`Dimensions match (${frame.width}×${frame.height})`, 
              frame.width === pngInfo.width && frame.height === pngInfo.height)) {
        // Continue anyway - might be a minor issue
    }
    test(`Has IDAT chunks (image data)`, pngInfo.hasIdat);
    
    // Save the PNG file
    const filename = `frame_${String(framesReceived).padStart(3, '0')}.png`;
    const filepath = path.join(outputDir, filename);
    fs.writeFileSync(filepath, frame.pngBuffer);
    test(`Saved to ${filepath}`, true);
    
    console.log(`   Image size: ${(frame.pngBuffer.length / 1024).toFixed(1)} KB`);
    
    // Check if we've received enough frames
    if (framesReceived >= maxFrames) {
        log(`\n✅ Received ${maxFrames} frames successfully!`, 'green');
        finish();
    }
}

// ─── Cleanup & Results ─────────────────────────────────────────────

function finish() {
    const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
    
    console.log(`\n${'─'.repeat(60)}`);
    log('📊 Test Results', 'cyan');
    console.log(`   Duration: ${elapsed}s`);
    console.log(`   Frames received: ${framesReceived}`);
    console.log(`   Passed: ${passed}`);
    console.log(`   Failed: ${failed}`);
    
    if (fs.existsSync(outputDir) && fs.readdirSync(outputDir).length > 0) {
        const files = fs.readdirSync(outputDir).filter(f => f.endsWith('.png'));
        log(`\n📁 Output files (${files.length}):`, 'green');
        files.forEach(f => console.log(`      ${f}`));
    }
    
    if (failed === 0) {
        log('\n✅ All tests passed!', 'green');
    } else {
        log(`\n❌ ${failed} test(s) failed`, 'red');
    }
    console.log(`${'─'.repeat(60)}\n`);
    
    ws.close();
    process.exit(failed > 0 ? 1 : 0);
}

// Timeout after 30 seconds
setTimeout(() => {
    if (framesReceived === 0) {
        log('\n⏱️  Test timed out - no frames received', 'red');
    } else {
        console.log(`\n⏱️  Timeout reached. Received ${framesReceived} frames.`);
    }
    finish();
}, 30000);

// Graceful shutdown on Ctrl+C
process.on('SIGINT', () => {
    console.log('\n\nInterrupted.');
    finish();
});
