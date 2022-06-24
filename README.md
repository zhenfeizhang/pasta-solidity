# Pasta curves on solidity

Solidity implementation of Pasta curves, with test vectors from Arkworks.

## Test

- `nix-shell`
- `hivemind & pasta-test-all`

## Gas cost

- `on curve testing`: 22724
- `doubling`: 24812
- `addition`: 26155
- `scalar mul`: 940148 (variable depending on the bits of scalars)
