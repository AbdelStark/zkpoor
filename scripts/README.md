# zkpoor Scripts

Utility scripts for the zkpoor project.

## find-demo-utxos.js

**Purpose**: Finds meaningful P2PKH UTXOs from recent Bitcoin blocks to use in our hackathon demo.

### What it does:
- 🔍 Searches recent Bitcoin blocks using mempool.space API
- 📋 Filters for P2PKH addresses (classic Bitcoin addresses starting with '1')
- 💰 Finds UTXOs with meaningful amounts (configurable, default > 0.001 BTC)
- ✅ Verifies UTXOs are still unspent
- 📄 Generates both JSON and Markdown output files

### Usage:
```bash
node scripts/find-demo-utxos.js
```

### Configuration:
The script can be easily configured by modifying the constructor values:

```javascript
this.minBlockHeight = 913139;  // Only search blocks after this height
this.minAmount = 0.001;        // Minimum BTC amount (0.001 = 100k sats)
this.targetCount = 3;          // Number of UTXOs to find
```

### Output Files:
- `demo-utxos.json` - Complete structured data for backend integration
- `demo-utxos.md` - Human-readable markdown summary

### API Rate Limiting:
- Built-in delays between requests to respect mempool.space API limits
- Configurable request delay (default 1 second)
- Limits transactions checked per block to avoid rate limits

### Example Output:
The script found these UTXOs for our demo:

1. **153Dra8VyKVvA7CBK7NPTh4CDL5ejPGfch** - 0.00870000 BTC (870,000 sats)
2. **1PnsRhYv3mYGe7EKuURACUzjRabcHuaRT** - 0.02335259 BTC (2,335,259 sats)  
3. **12EunTsM6Q7axmi5BjJWya8fYx3TSEejQ6** - 0.00143794 BTC (143,794 sats)

**Total**: 0.03349053 BTC (3,349,053 sats)

Perfect amounts for demonstrating zkpoor's proof generation capabilities! 🚀

## Technical Notes

- Uses Node.js with ES modules
- Makes HTTPS requests to mempool.space API
- Implements proper error handling and retry logic  
- Respects API rate limits with built-in delays
- Validates Bitcoin addresses and UTXO formats
- Checks that UTXOs are actually unspent before including them

The found UTXOs are real, unspent outputs that can be used to demonstrate the complete zkpoor workflow from UTXO input through proof generation and verification.