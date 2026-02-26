# limitless-cli

Command-line interface for [Limitless Exchange](https://limitless.exchange) — browse prediction markets, place orders, manage positions, and interact with on-chain contracts on Base.

> **Limitless Exchange docs**: [docs.limitless.exchange](https://docs.limitless.exchange) &nbsp;|&nbsp; **API reference**: [limitless.mintlify.app](https://limitless.mintlify.app)

## Installation

### Homebrew

```bash
brew install limitless-labs-group/limitless-cli/limitless
```

### From source

```bash
git clone https://github.com/limitless-labs-group/limitless-cli.git
cd limitless-cli
cargo build --release
cp target/release/limitless ~/.cargo/bin/
```

Requires Rust 1.75+.

## Quick Start

```bash
# 1. Run the interactive setup wizard (saves API key + private key to config)
limitless setup

# 2. Browse markets
limitless markets list
limitless markets search "bitcoin"

# 3. Check an orderbook
limitless orderbook book btc-above-100000-0228

# 4. Approve tokens for a market, then trade
limitless approve set --slug btc-above-100000-0228

# 5a. Place a limit order (GTC)
limitless trading create --slug btc-above-100000-0228 --side buy --outcome yes --price 0.65 --size 100

# 5b. Or place a market order (FOK) — spend 50 USDC at best price
limitless trading create --slug btc-above-100000-0228 --side buy --outcome yes --size 50 --order-type FOK
```

Get your API key from [limitless.exchange](https://limitless.exchange) → Profile → Api keys.

`limitless setup` prompts for your API key and private key, then saves them to `~/.config/limitless/config.json`. Run it again anytime to update your credentials.

## Configuration

Configuration is stored at `~/.config/limitless/config.json`:

```json
{
  "api_key": "lmts_...",
  "private_key": "0x...",
  "chain_id": 8453,
  "rpc_url": "https://mainnet.base.org",
  "api_url": "https://api.limitless.exchange",
  "ws_url": "wss://ws.limitless.exchange"
}
```

**Auth resolution priority** (for both API key and private key):

1. CLI flag (`--api-key` / `--private-key`)
2. Environment variable (`LIMITLESS_API_KEY` / `LIMITLESS_PRIVATE_KEY`)
3. Config file

## Commands

### Global Flags

All commands accept these flags (can appear before or after the subcommand):

```
--output table|json    Output format (default: table)
--api-key <key>        Override API key
--private-key <key>    Override private key
```

Short forms: `-o json`, `-o table`.

### `markets` — Browse and search prediction markets

```bash
limitless markets list                              # List active markets (page 0, 20 results)
limitless markets list --sort-by trending           # Sort by trending
limitless markets list --sort-by newest             # Sort by newest
limitless markets list --sort-by ending-soon        # Ending soonest first
limitless markets list --sort-by high-value         # Highest volume first
limitless markets list --sort-by lp-rewards         # LP reward markets
limitless markets list --trade-type clob            # Filter to CLOB markets only
limitless markets list --trade-type amm             # Filter to AMM markets only
limitless markets list --category 28                # Filter by category ID
limitless markets list --page 2 --limit 10          # Paginate (page 2, 10 per page)
limitless markets get <slug>                        # Full market details
limitless markets search "btc"                      # Search markets
limitless markets search "eth" --limit 5            # Search with max results
limitless markets slugs                             # List all active slugs
limitless markets categories                        # List categories with market counts
```

Use `markets categories` to find category IDs, then filter with `--category <id>`.

### `orderbook` — View orderbook, prices, and spreads

```bash
limitless orderbook book <slug>                     # Full orderbook (bids + asks)
limitless orderbook price <slug>                    # Best bid & ask with sizes and spread
limitless orderbook midpoint <slug>                 # Midpoint price
limitless orderbook spread <slug>                   # Bid-ask spread
limitless orderbook last-trade <slug>               # Last trade price
limitless orderbook history <slug>                  # Historical prices
limitless orderbook events <slug>                   # Market feed events
limitless orderbook monitor <slug>                  # Live orderbook monitor (TUI)
```

#### `orderbook monitor` — Live TUI

`limitless orderbook monitor <slug>` opens a full-screen terminal UI that streams real-time orderbook updates via WebSocket. Features:

- **Live bids/asks** — color-coded price levels with cumulative depth
- **Midpoint & spread** — recalculated on every update
- **VWAP at dollar depths** — shows average fill price for hypothetical $10, $50, $100, and $200 market orders on both buy and sell sides
- **Connection status** — shows update count and time since last update

Press `q` or `Esc` to exit.

### `trading` — Place and manage orders

```bash
# Place a GTC limit order (rests on book until filled or cancelled)
limitless trading create \
  --slug <slug> \
  --side buy \
  --outcome yes \
  --price 0.65 \
  --size 100

# FOK market buy — spend 50 USDC at best available price
limitless trading create \
  --slug <slug> \
  --side buy \
  --outcome yes \
  --size 50 \
  --order-type FOK

# FOK market sell — sell 100 shares at best available price
limitless trading create \
  --slug <slug> \
  --side sell \
  --outcome yes \
  --size 100 \
  --order-type FOK

# View and manage orders
limitless trading orders <slug>                     # List your orders
limitless trading orders <slug> -s LIVE             # Filter by status
limitless trading locked-balance <slug>             # Check locked balance
limitless trading cancel <order-id>                 # Cancel one order
limitless trading cancel-batch id1,id2,id3          # Cancel multiple
limitless trading cancel-all <slug>                 # Cancel all for market
```

**Order types:**
- `GTC` (Good Till Cancelled) — default limit order. Requires `--price` (0.01–0.99). Rests on the book until filled or cancelled.
- `FOK` (Fill or Kill) — market order. No `--price` needed. Fills immediately at best available price or cancels entirely.
  - **Buy**: `--size` = USDC amount to spend
  - **Sell**: `--size` = number of shares to sell

Order signing uses **EIP-712** (domain: "Limitless CTF Exchange", chain ID 8453). Both API key and private key are required. See the [Limitless API docs](https://limitless.mintlify.app/api-reference/trading/create-order) for full details.

### `portfolio` — View your positions and PnL

```bash
limitless portfolio positions                       # All positions (table)
limitless portfolio positions --status funded       # Only active/open positions
limitless portfolio positions --status resolved     # Only resolved/closed positions
limitless portfolio trades                          # Trade history (table)
limitless portfolio pnl                             # PnL summary
limitless portfolio pnl --timeframe 7d              # PnL over 7 days
limitless portfolio pnl --timeframe 30d             # PnL over 30 days
limitless portfolio history                         # Portfolio history (table)
limitless portfolio history --page 1 --limit 25     # Paginated history
limitless portfolio points                          # Accumulated points breakdown
limitless portfolio allowance                       # Trading allowance status
limitless portfolio allowance -t clob               # CLOB allowance specifically
```

### `profiles` — Public portfolio data for any address

```bash
limitless profiles positions <address>              # Public positions
limitless profiles volume <address>                 # Traded volume stats
limitless profiles pnl <address>                    # PnL chart
limitless profiles pnl <address> --timeframe 30d    # PnL over 30 days
```

### `approve` — On-chain token approvals

Before trading on a market, you need to approve USDC and Conditional Tokens for the exchange contract.

```bash
limitless approve check --slug <slug>               # Check approval status
limitless approve set --slug <slug>                 # Set all approvals
```

`approve set` sends up to 3 transactions:
1. USDC `approve(exchange, MAX)`
2. Conditional Token `setApprovalForAll(exchange, true)`
3. Conditional Token `setApprovalForAll(adapter, true)` — for NegRisk markets only

### `wallet` — Key management

```bash
limitless wallet create                             # Generate new wallet
limitless wallet import <private-key>               # Import existing key
limitless wallet show                               # Show wallet info
limitless wallet address                            # Print address only
limitless wallet reset                              # Clear stored keys
```

### `shell` — Interactive REPL

```bash
limitless shell
```

Opens an interactive prompt where you can run commands without the `limitless` prefix:

```
limitless> markets list
limitless> orderbook book btc-above-100000-0228
limitless> exit
```

Global flags (`--output`, `--api-key`, `--private-key`) are inherited from the shell invocation.

## Output Formats

All commands support `--output table` (default) and `--output json`:

```bash
limitless markets list                              # Human-readable table
limitless markets list --output json                # JSON (for scripting)
limitless markets list -o json                      # Short form
limitless portfolio positions -o json               # Works after any subcommand
```

## Chain Details

| | |
|---|---|
| **Chain** | Base (chain ID 8453) |
| **RPC** | `https://mainnet.base.org` |
| **USDC** | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` |
| **Conditional Tokens** | `0xC9c98965297Bc527861c898329Ee280632B76e18` |

## Resources

- [Limitless Exchange](https://limitless.exchange) — the platform
- [Documentation](https://docs.limitless.exchange) — guides and concepts
- [API Reference](https://limitless.mintlify.app) — REST API endpoints
- [TypeScript SDK](https://github.com/limitless-labs-group/limitless-exchange-ts-sdk) — official TS SDK

## License

MIT
