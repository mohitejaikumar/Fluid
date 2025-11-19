# Fluid - DeFi Yield Aggregator

A Solana-based DeFi yield aggregator that optimizes deposits across multiple lending protocols (Juplend and Kamino) to maximize returns for users.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Local Development Setup](#local-development-setup)
- [Project Structure](#project-structure)
- [Key Features](#key-features)

## Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** (v16 or higher)
- **Yarn** or **npm**
- **Rust** (latest stable version)
- **Anchor CLI** (v0.30.0 or higher version)
- **Solana CLI** (v1.18.0 or higher)

## Local Development Setup

### Step 1: Install Surfpool

Surfpool is a drop-in replacement for `solana-test-validator` that allows you to simulate programs locally using Mainnet accounts fetched just in time.

#### Install via npm (recommended):

```bash
npm install -g @txtx/surfpool
```

#### Or via Homebrew (macOS):

```bash
brew install txtx/tap/surfpool
```

#### Verify installation:

```bash
surfpool --version
```

For more installation options and details, visit the [official Surfpool documentation](https://docs.surfpool.run/).

### Step 2: Build the Anchor Program

Navigate to the contract directory and build the project:

```bash
cd contract
anchor build
```

This will compile the Solana program and generate the necessary artifacts in the `target/` directory.

### Step 3: Start Surfpool with Watch Mode

Start the local Surfpool validator with watch mode enabled:

```bash
surfpool start --watch
```

**What happens here:**
- Surfpool will start a local Solana validator (Surfnet)
- It will automatically deploy your program
- Watch mode will rebuild and redeploy on file changes
- Wait for the output confirming successful program deployment

You should see output similar to:
```
✓ Surfpool started successfully
✓ Program deployed: <program_id>
```

### Step 4: Fund Your Local Wallet with USDC

While Surfpool is running, open a new terminal window and fund your local wallet with USDC tokens on Surfnet.

#### Get your wallet address:

```bash
solana address
```

#### Fund with SOL (for transaction fees):

```bash
solana airdrop 10
```

#### Fund with USDC:

Since Surfpool simulates Mainnet conditions, you can use Surfpool's cheatcodes to mint USDC tokens to your wallet:

```bash
# Replace AR17nM5Ny8kVyHZMhyULaecb1K9HdQcN7VPNjyPigdTL with your actual wallet address
# EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v is the USDC mint address
# TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA is the SPL Token program

curl -X POST http://localhost:8899 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "surfnet_setTokenAccount",
    "params": [
      "AR17nM5Ny8kVyHZMhyULaecb1K9HdQcN7VPNjyPigdTL",
      "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "amount": 100000000000
    ],
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
  }'
```

This will mint 100,000 USDC (100000000000 in smallest units with 6 decimals) to your wallet.

### Step 5: Run Tests

With Surfpool running and your wallet funded, execute the test suite:

```bash
anchor test
```


**What the tests do:**
- Initialize the aggregator configuration
- Test deposit functionality across protocols
- Test withdrawal operations
- Test rebalancing logic
- Verify strategy updates

### Step 6: View Events and Verify Transactions

All test transactions will output signatures to the console. You can verify transactions using:


#### View events in test output:

The Anchor test framework will automatically decode and display program events in the console output. Look for events like:
- `DepositEvent`
- `WithdrawEvent`
- `RebalanceEvent`
- `StrategyUpdateEvent`

#### Use Surfpool Studio (Optional):

Surfpool provides a local web UI for enhanced debugging:

```bash
# Access Surfpool Studio at http://127.0.0.1:18488/
```

Surfpool Studio allows you to:
- View all transactions and accounts
- Inspect program state
- Debug events in real-time
- Generate custom APIs from your smart contracts

## Project Structure

```
Fluid/
├── contract/
│   ├── programs/contract/src/
│   │   ├── instructions/       # Program instructions
│   │   │   ├── deposit.rs
│   │   │   ├── withdraw.rs
│   │   │   ├── rebalance.rs
│   │   │   └── ...
│   │   ├── helpers/            # Helper functions
│   │   │   ├── juplend/        # Juplend protocol integration
│   │   │   ├── kamino/         # Kamino protocol integration
│   │   │   └── ...
│   │   ├── states/             # Program state definitions
│   │   └── lib.rs              # Program entry point
│   ├── tests/                  # Test files
│   ├── IDLs/                   # External protocol IDLs
│   └── target/                 # Build artifacts
└── README.md
```

## Key Features

- **Multi-Protocol Support**: Integrates with Juplend and Kamino lending protocols
- **Automatic Rebalancing**: Optimizes fund allocation across protocols
- **Share-based System**: Users receive shares representing their proportional ownership
- **Flexible Strategies**: Configurable allocation strategies for different risk profiles
- **Decoupled Implementation**: Easily you can integrate new Protocol with minimal change in codebase



---

**Happy Building!

