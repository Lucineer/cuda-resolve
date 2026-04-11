# cuda-resolve

A2A deliberative compilation in Rust — JSON payloads, confidence propagation, multi-agent protocol

Part of the Cocapn fleet — a Lucineer vessel component.

## What It Does

### Key Types

- `Payload` — core data structure
- `PayloadChain` — core data structure
- `BaseAgent` — core data structure
- `IntentParser` — core data structure
- `DeliberationEngine` — core data structure
- `TraceEntry` — core data structure
- _and 1 more (see source)_

## Quick Start

```bash
# Clone
git clone https://github.com/Lucineer/cuda-resolve.git
cd cuda-resolve

# Build
cargo build

# Run tests
cargo test
```

## Usage

```rust
use cuda_resolve::*;

// See src/lib.rs for full API
// 5 unit tests included
```

### Available Implementations

- `Payload` — see source for methods
- `PayloadChain` — see source for methods
- `BaseAgent` — see source for methods
- `IntentParser` — see source for methods
- `Agent for IntentParser` — see source for methods
- `DeliberationEngine` — see source for methods

## Testing

```bash
cargo test
```

5 unit tests covering core functionality.

## Architecture

This crate is part of the **Cocapn Fleet** — a git-native multi-agent ecosystem.

- **Category**: other
- **Language**: Rust
- **Dependencies**: See `Cargo.toml`
- **Status**: Active development

## Related Crates


## Fleet Position

```
Casey (Captain)
├── JetsonClaw1 (Lucineer realm — hardware, low-level systems, fleet infrastructure)
├── Oracle1 (SuperInstance — lighthouse, architecture, consensus)
└── Babel (SuperInstance — multilingual scout)
```

## Contributing

This is a fleet vessel component. Fork it, improve it, push a bottle to `message-in-a-bottle/for-jetsonclaw1/`.

## License

MIT

---

*Built by JetsonClaw1 — part of the Cocapn fleet*
*See [cocapn-fleet-readme](https://github.com/Lucineer/cocapn-fleet-readme) for the full fleet roadmap*
