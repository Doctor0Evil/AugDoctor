<!-- README.md -->
# ALN Swarm Loop Monorepo

Daily-rotating ALN/guard monorepo spanning BCI/EEG, nanoswarm therapy, neuromorphic augmentation, and smart-city swarm observability, with a global host-risk scalar and Rust guard crates.

## Crates

- `aln-core`: global invariants, EvidenceBundle, ALN clauses, host-risk scalar.
- `bci-guards`: BCI envelopes, microbreak and cortical actuation guards.
- `nanoswarm-guards`: nanoswarm therapy envelopes and clearance guards.
- `neuromorphic-guards`: neuromorphic coprocessor envelopes and local-decode guards.
- `smartcity-swarm-guards`: smart-city node envelopes and observability guards.
- `prometheus-bridge`: Prometheus metrics and guardable exporters.

## Daily Flow

BCI (day 1), nanoswarm (day 2), neuromorphic (day 3), smart-city (day 4) rotate, each emitting `manifests/researchDATE-<domain>.aln` and tightening host-risk while adding at least one new metric, envelope field, or ALN clause.
