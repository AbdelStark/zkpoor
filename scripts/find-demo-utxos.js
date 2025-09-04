#!/usr/bin/env node

/**
 * UTXO Finder for zkpoor Demo
 * 
 * This script finds meaningful P2PKH UTXOs from recent blocks
 * to use in our zkpoor hackathon demo.
 * 
 * Requirements:
 * - P2PKH outputs (classic Bitcoin addresses starting with 1)
 * - After block height 913139
 * - Meaningful amounts (ideally > 0.01 BTC)
 * - Different addresses to show variety
 */

import https from 'https';
import fs from 'fs';
import { fileURLToPath } from 'url';
import path from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class UTXOFinder {
    constructor() {
        this.baseURL = 'mempool.space';
        this.minBlockHeight = 913139;
        this.minAmount = 0.001; // 0.001 BTC minimum (100k sats)
        this.targetCount = 3;
        this.foundUTXOs = [];
        this.checkedAddresses = new Set();
        this.requestDelay = 1000; // 1 second between requests to be nice to API
    }

    async sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    async makeRequest(path) {
        return new Promise((resolve, reject) => {
            const options = {
                hostname: this.baseURL,
                port: 443,
                path: path,
                method: 'GET',
                headers: {
                    'User-Agent': 'zkpoor-demo-utxo-finder/1.0'
                }
            };

            const req = https.request(options, (res) => {
                let data = '';
                res.on('data', (chunk) => {
                    data += chunk;
                });
                res.on('end', () => {
                    try {
                        // Some endpoints return plain text, others return JSON
                        if (data.trim().startsWith('{') || data.trim().startsWith('[')) {
                            const jsonData = JSON.parse(data);
                            resolve(jsonData);
                        } else {
                            // Plain text response (like block hash)
                            resolve(data.trim());
                        }
                    } catch (e) {
                        console.log('Raw response:', data.slice(0, 200));
                        reject(new Error(`Failed to parse response: ${e.message}`));
                    }
                });
            });

            req.on('error', (e) => {
                reject(e);
            });

            req.setTimeout(30000, () => {
                req.destroy();
                reject(new Error('Request timeout'));
            });

            req.end();
        });
    }

    isP2PKH(address) {
        // P2PKH addresses start with '1' and are 26-35 characters
        return address && address.startsWith('1') && address.length >= 26 && address.length <= 35;
    }

    satsToBTC(sats) {
        return sats / 100000000;
    }

    async getCurrentBlockHeight() {
        console.log('🔍 Getting current block height...');
        try {
            const tipHeight = await this.makeRequest('/api/blocks/tip/height');
            console.log(`📊 Current block height: ${tipHeight}`);
            return tipHeight;
        } catch (error) {
            console.error('❌ Failed to get current block height:', error.message);
            throw error;
        }
    }

    async getBlockTransactions(blockHash) {
        console.log(`🔍 Getting transactions for block ${blockHash.slice(0, 10)}...`);
        try {
            const txids = await this.makeRequest(`/api/block/${blockHash}/txids`);
            return txids.slice(0, 20); // Limit to first 20 transactions per block to avoid rate limits
        } catch (error) {
            console.log(`⚠️ Failed to get transactions for block ${blockHash}: ${error.message}`);
            return [];
        }
    }

    async getTransaction(txid) {
        try {
            const tx = await this.makeRequest(`/api/tx/${txid}`);
            await this.sleep(100); // Small delay between transaction requests
            return tx;
        } catch (error) {
            console.log(`⚠️ Failed to get transaction ${txid}: ${error.message}`);
            return null;
        }
    }

    async analyzeTransaction(tx) {
        if (!tx || !tx.vout) return [];

        const utxos = [];
        for (let i = 0; i < tx.vout.length; i++) {
            const output = tx.vout[i];
            
            // Skip if no address or not P2PKH
            if (!output.scriptpubkey_address || !this.isP2PKH(output.scriptpubkey_address)) {
                continue;
            }

            // Skip if amount too small
            const btcAmount = this.satsToBTC(output.value);
            if (btcAmount < this.minAmount) {
                continue;
            }

            // Skip if we already have a UTXO from this address
            if (this.checkedAddresses.has(output.scriptpubkey_address)) {
                continue;
            }

            // Check if UTXO is still unspent
            try {
                const outspend = await this.makeRequest(`/api/tx/${tx.txid}/outspend/${i}`);
                await this.sleep(100);
                
                if (!outspend.spent) {
                    utxos.push({
                        txid: tx.txid,
                        vout: i,
                        value: output.value,
                        address: output.scriptpubkey_address,
                        scriptPubKey: output.scriptpubkey,
                        btcAmount: btcAmount,
                        blockHeight: tx.status?.block_height || 'mempool',
                        blockTime: tx.status?.block_time || Date.now() / 1000
                    });
                    
                    this.checkedAddresses.add(output.scriptpubkey_address);
                }
            } catch (error) {
                console.log(`⚠️ Failed to check outspend for ${tx.txid}:${i}: ${error.message}`);
            }
        }

        return utxos;
    }

    async searchBlockForUTXOs(blockHeight) {
        console.log(`🔍 Searching block ${blockHeight} for UTXOs...`);
        
        try {
            // Get block hash from block height
            const blockHash = await this.makeRequest(`/api/block-height/${blockHeight}`);
            await this.sleep(this.requestDelay);
            
            // Get transactions from this block
            const txids = await this.getBlockTransactions(blockHash);
            console.log(`📦 Found ${txids.length} transactions in block ${blockHeight}`);

            for (let j = 0; j < Math.min(txids.length, 10); j++) { // Limit to 10 transactions per block
                if (this.foundUTXOs.length >= this.targetCount) break;

                const tx = await this.getTransaction(txids[j]);
                if (!tx) continue;

                const utxos = await this.analyzeTransaction(tx);
                for (const utxo of utxos) {
                    console.log(`✅ Found UTXO: ${utxo.btcAmount.toFixed(8)} BTC at ${utxo.address}`);
                    this.foundUTXOs.push(utxo);
                    
                    if (this.foundUTXOs.length >= this.targetCount) break;
                }

                await this.sleep(500); // Delay between transactions
            }

        } catch (error) {
            console.log(`⚠️ Failed to search block ${blockHeight}: ${error.message}`);
        }
    }

    async findDemoUTXOs() {
        console.log('🚀 Starting UTXO search for zkpoor demo...');
        console.log(`📋 Criteria: P2PKH addresses, min ${this.minAmount} BTC, after block ${this.minBlockHeight}`);
        
        try {
            const currentHeight = await this.getCurrentBlockHeight();
            
            // Search recent blocks starting from a reasonable point
            const startHeight = Math.max(this.minBlockHeight, currentHeight - 100);
            console.log(`🎯 Searching blocks from ${startHeight} to ${currentHeight}`);

            for (let height = currentHeight; height >= startHeight && this.foundUTXOs.length < this.targetCount; height -= 1) {
                await this.searchBlockForUTXOs(height);
                
                // Add delay between blocks to be respectful to the API
                await this.sleep(this.requestDelay);
            }

            return this.foundUTXOs;

        } catch (error) {
            console.error('❌ Failed to find UTXOs:', error);
            throw error;
        }
    }

    formatUTXOsForDemo(utxos) {
        console.log('\n🎉 Demo UTXOs Found:');
        console.log('='.repeat(60));
        
        const formatted = utxos.map((utxo, index) => {
            console.log(`\n${index + 1}. Address: ${utxo.address}`);
            console.log(`   TXID: ${utxo.txid}`);
            console.log(`   Vout: ${utxo.vout}`);
            console.log(`   Amount: ${utxo.btcAmount.toFixed(8)} BTC (${utxo.value} sats)`);
            console.log(`   Script: ${utxo.scriptPubKey}`);
            console.log(`   Block: ${utxo.blockHeight}`);
            
            return {
                address: utxo.address,
                txid: utxo.txid,
                vout: utxo.vout,
                amount_btc: utxo.btcAmount,
                amount_sats: utxo.value,
                scriptPubKey: utxo.scriptPubKey,
                blockHeight: utxo.blockHeight,
                description: `Demo UTXO ${index + 1}: ${utxo.btcAmount.toFixed(8)} BTC`
            };
        });

        return formatted;
    }

    async saveResults(utxos) {
        const results = {
            timestamp: new Date().toISOString(),
            criteria: {
                minBlockHeight: this.minBlockHeight,
                minAmount: this.minAmount,
                addressType: 'P2PKH'
            },
            utxos: this.formatUTXOsForDemo(utxos),
            total_btc: utxos.reduce((sum, utxo) => sum + utxo.btcAmount, 0),
            total_sats: utxos.reduce((sum, utxo) => sum + utxo.value, 0)
        };

        const filename = 'demo-utxos.json';
        fs.writeFileSync(filename, JSON.stringify(results, null, 2));
        console.log(`\n💾 Results saved to ${filename}`);
        
        // Also create a simple markdown file
        const mdContent = `# Demo UTXOs for zkpoor

Generated on: ${results.timestamp}

## Criteria
- Address Type: P2PKH (starts with '1')
- Minimum Amount: ${this.minAmount} BTC
- After Block Height: ${this.minBlockHeight}

## UTXOs Found

${results.utxos.map((utxo, i) => `
### UTXO ${i + 1}
- **Address**: \`${utxo.address}\`
- **TXID**: \`${utxo.txid}\`
- **Vout**: \`${utxo.vout}\`
- **Amount**: ${utxo.amount_btc.toFixed(8)} BTC (${utxo.amount_sats.toLocaleString()} sats)
- **Script**: \`${utxo.scriptPubKey}\`
- **Block Height**: ${utxo.blockHeight}
`).join('')}

## Summary
- **Total UTXOs**: ${results.utxos.length}
- **Total Amount**: ${results.total_btc.toFixed(8)} BTC (${results.total_sats.toLocaleString()} sats)
`;

        fs.writeFileSync('demo-utxos.md', mdContent);
        console.log('📝 Markdown summary saved to demo-utxos.md');
        
        return results;
    }
}

// Main execution
async function main() {
    const finder = new UTXOFinder();
    
    try {
        console.log('⚡ zkpoor Demo UTXO Finder');
        console.log('🔍 Searching for meaningful P2PKH UTXOs...\n');
        
        const utxos = await finder.findDemoUTXOs();
        
        if (utxos.length === 0) {
            console.log('😞 No suitable UTXOs found. Try adjusting the criteria.');
            process.exit(1);
        }
        
        const results = await finder.saveResults(utxos);
        
        console.log('\n🎯 Success! Found UTXOs ready for demo');
        console.log(`💰 Total: ${results.total_btc.toFixed(8)} BTC across ${results.utxos.length} UTXOs`);
        console.log('\nFiles generated:');
        console.log('  📄 demo-utxos.json - Full data for backend');
        console.log('  📝 demo-utxos.md - Human-readable summary');
        
    } catch (error) {
        console.error('\n❌ Script failed:', error.message);
        process.exit(1);
    }
}

// Check if this is the main module
if (import.meta.url === `file://${process.argv[1]}`) {
    main();
}

export default UTXOFinder;