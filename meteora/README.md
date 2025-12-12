# meteora Substreams modules

This package was initialized via `substreams init`, using the `sol-hello-world` template.

## Usage

```bash
substreams build
substreams auth
substreams gui       			  # Get streaming!
```

Optionally, you can publish your Substreams to the [Substreams Registry](https://substreams.dev).

```bash
substreams registry login         # Login to substreams.dev
substreams registry publish       # Publish your Substreams to substreams.dev
```

## Modules

### `meteora`

Consumes Solana `blocks_without_votes` from `solana-common`, applies constant filters (defined in `src/lib.rs`) for `program`/`account`, and emits matching transactions as `MyData`.
