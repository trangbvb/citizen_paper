# citizen_paper

## Project Title
citizen_paper

## Project Description
citizen_paper is an on-chain digital citizenship credential registry built
on Stellar using Soroban smart contracts. In many real-world flows —
exchanges, banks, DeFi front-ends, travel platforms — a service needs to
know *whether* a user is a verified citizen, *under which jurisdiction*
they were verified, and *whether that status is still current*. Today this
is done with siloed KYC providers, repeated paperwork, and opaque APIs.
citizen_paper replaces that with a single shared ledger: a registered
government authority issues a tamper-evident credential once, and any
verifier on the network can read its current status with a single call.

## Project Vision
The long-term goal is a portable, jurisdiction-aware digital identity
primitive that lives on public infrastructure rather than inside any
single vendor. A citizen should be able to prove their status to any
counterparty in milliseconds, without re-enrolling, and a government
should be able to revoke or suspend that status in one place for it to
take effect everywhere. citizen_paper is the first, minimal building
block: one contract, one authority per deployment, and a tiny, stable
status interface that other dApps and off-chain services can build on
top of.

## Key Features
- **Single-call issuance** — a registered authority mints a credential
  for a citizen in one transaction, recording jurisdiction, issuance
  time, and `valid_until` on-chain.
- **Five-state verification** — `verify` returns a stable `u32` code
  (`0` none, `1` valid, `2` expired, `3` revoked, `4` suspended), making
  it trivial for off-chain verifiers to branch on status.
- **Authority-gated mutations** — every state change goes through
  `require_auth` and a stored-authority check, so only the originally
  registered government address can issue, revoke, suspend, or
  reinstate.
- **Time-aware expiry** — `valid_until` is checked against the ledger
  timestamp at verification time, so a credential never reports valid
  after its expiry without an explicit authority action.
- **Audit-friendly reasons** — `revoke` and `suspend` accept a
  `Symbol` reason that is stored alongside the credential for later
  inspection.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** identity dApp — see `contracts/citizen_paper/src/lib.rs` for the full citizen_paper business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** CDXN5QVGT3KB6W7GUTSWZIWERTPC2GLNSRIUUMERSI2QOLXFHK5LBAKQ
- **Explorer template:** https://stellar.expert/explorer/testnet/tx/eb71c16c8732278d859d74386ed03652838c79ff88290f90ddcdd44311ea560f
- **Screenshot of deployed contract on Stellar Expert:**
![screenshot](https://ibb.co/7xx55Dc8)


## Future Scope
- **Multi-authority and delegation** — support a council or federation
  of authorities, with per-jurisdiction signing keys and quorum rules.
- **Selective disclosure** — add zero-knowledge attestations so a
  citizen can prove *that* they are valid to a verifier without
  revealing the underlying address, mirroring real-world ZK-KYC work.
- **Credential renewal and upgrade paths** — let `reinstate` be paired
  with a renewal handshake so expired credentials can be refreshed
  without a fresh `issue` call.
- **Off-chain indexer SDK** — ship a small client library that
  consumes `verify` results and the `reason` field so auditors and
  compliance teams can reconstruct the lifecycle of any credential.
- **Mainnet and cross-chain anchoring** — graduate the registry from
  Testnet to a permissioned Mainnet deployment and periodically anchor
  a Merkle root of issued credentials to a public chain for
  additional trust minimisation.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `citizen_paper` (identity)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
