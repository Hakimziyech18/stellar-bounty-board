# Contributor Onboarding Guide

Welcome to **Stellar Bounty Board** a contribution-focused Stellar MVP for open source maintainers. This guide will get you from zero to a running local environment, help you understand the codebase, and point you toward your first meaningful contribution.

---

## Table of Contents

1. [What Is This Project?](#1-what-is-this-project)
2. [Quick Visual Overview](#2-quick-visual-overview)
3. [Prerequisites](#3-prerequisites)
4. [Getting the Code](#4-getting-the-code)
5. [Running the Project Locally](#5-running-the-project-locally)
6. [Repository Structure](#6-repository-structure)
7. [Where to Make Common Changes](#7-where-to-make-common-changes)
8. [Understanding the API](#8-understanding-the-api)
9. [Testing Your Changes](#9-testing-your-changes)
10. [Picking Your First Issue](#10-picking-your-first-issue)
11. [Issue Types Explained](#11-issue-types-explained)
12. [Submitting a Pull Request](#12-submitting-a-pull-request)
13. [Architecture & Deployment Links](#13-architecture--deployment-links)
14. [Getting Help](#14-getting-help)
15. [Stellar & Soroban Glossary](#15-stellar--soroban-glossary)

---

## 1. What Is This Project?

Stellar Bounty Board lets open source maintainers fund GitHub issues as Stellar bounties. Contributors can reserve work, submit a PR link, and receive a payout — all tracked through a React dashboard, a Node.js/Express backend, and a Soroban smart contract scaffold.

**Current MVP flow:**

```
Maintainer creates bounty → Contributor reserves it
→ Contributor submits PR link → Maintainer releases payout (or refunds)
```

The backend uses JSON file persistence today, with a clear path to Postgres. The Soroban contract models the same lifecycle on-chain, ready to become the source of truth once wallet authentication is wired up.

> [!NOTE]
> **New to Stellar or Soroban?** Terms like _Soroban_, _XLM_, _escrow_, _SAC_, _XDR_, and _Horizon_ appear throughout these docs. Jump to the [Stellar & Soroban Glossary](#15-stellar--soroban-glossary) at any time for plain-English definitions and links to the official reference.

---

## 2. Quick Visual Overview

### Demo Video

> [!NOTE]
> [Watch the 2-minute project walkthrough here (Coming Soon)](#)

### System Architecture

This 3-layer diagram shows how the Dashboard, API, and Soroban Contract interact.

```mermaid
graph TD
    subgraph Client
        A[Frontend - React/Vite]
    end
    subgraph Server
        B[Backend - Node/Express]
        C[(JSON Persistence)]
    end
    subgraph Blockchain
        D[Smart Contract - Soroban/Rust]
    end

    A -- REST API --> B
    B -- Reads/Writes --> C
    A -- Wallet/Tx --> D
    B -- Event Indexer (Planned) --> D
```

---

## 3. Prerequisites

Make sure you have the following installed before you begin:

| Tool                                                                                             | Minimum version | Why it's needed                             |
| ------------------------------------------------------------------------------------------------ | --------------- | ------------------------------------------- |
| [Node.js](https://nodejs.org/)                                                                   | 18+             | Frontend (Vite/React) and backend (Express) |
| npm                                                                                              | 9+              | Workspace scripts (`install:all`, `dev:*`)  |
| [Rust](https://www.rust-lang.org/tools/install)                                                  | stable (1.75+)  | Soroban contract compilation                |
| [Stellar CLI (`stellar`)](https://developers.stellar.org/docs/tools/developer-tools/stellar-cli) | latest          | Contract build, deploy, and invocation      |
| Git                                                                                              | any recent      | Cloning and branching                       |

> **Rust is only required if you plan to work on the smart contract.** Frontend and backend contributors do not need it.

---

```bash
# Fork the repo on GitHub, then clone your fork
git clone https://github.com/<your-username>/stellar-bounty-board.git
cd stellar-bounty-board

# Add the upstream remote so you can pull future changes
git remote add upstream https://github.com/ritik4ever/stellar-bounty-board.git
```

---

## 5. Running the Project Locally

### Install all dependencies (one command)

```bash
npm run install:all
```

This runs `npm install` in the root, `frontend/`, and `backend/` in one step.

### Start the backend

```bash
npm run dev:backend
```

The Express API starts at **http://localhost:3001**. It reads from and writes to `backend/data/bounties.json`.

### Start the frontend

Open a second terminal:

```bash
npm run dev:frontend
```

The React/Vite dashboard starts at **http://localhost:3000**. It proxies `/api` requests to the backend automatically — you do not need to configure CORS.

### Verify everything is working

```
http://localhost:3000          → Bounty dashboard UI
http://localhost:3001/api/health  → Should return { "status": "ok" }
http://localhost:3001/api/bounties → Should return [] (empty on first run)
```

### Build for production

```bash
npm run build
```

---

## 6. Repository Structure

```
stellar-bounty-board/
├── frontend/               # React + Vite dashboard
│   ├── src/
│   │   ├── components/     # Reusable UI components
│   │   ├── pages/          # Route-level views
│   │   ├── hooks/          # Custom React hooks
│   │   └── api/            # Typed fetch wrappers for the backend
│   └── vite.config.ts
│
├── backend/                # Node.js + Express REST API
│   ├── src/
│   │   ├── routes/         # Route handlers (bounties, health, open-issues)
│   │   ├── validators/     # Zod schemas for request validation
│   │   └── persistence/    # JSON read/write helpers
│   └── data/
│       └── bounties.json   # Live data file (gitignored in production)
│
├── contracts/              # Soroban Rust smart contract
│   ├── src/
│   │   └── lib.rs          # Contract entry points and escrow logic
│   └── Cargo.toml
│
├── docs/
│   └── issues/             # Pre-written issue drafts ready to open on GitHub
│
├── .github/
│   └── ISSUE_TEMPLATE/     # GitHub issue templates
│
├── CONTRIBUTING.md         # High-level contribution areas
├── ONBOARDING.md           # ← You are here
├── package.json            # Root workspace scripts
└── README.md
```

---

## 7. Where to Make Common Changes

### UI / Frontend changes

**Location:** `frontend/src/`

- To change how bounties are displayed → `components/` and `pages/`
- To add a new page or route → `pages/` + update the router
- To change how API calls are made → `src/api/`
- To add wallet connection UI → `components/` (new component) + hook in `hooks/`

**Stack:** React, TypeScript, Vite. Standard React patterns apply.

---

### Backend / API changes

**Location:** `backend/src/`

- To add a new endpoint → create a handler in `routes/` and register it in the Express app
- To change validation rules → update the Zod schema in `validators/`
- To swap JSON persistence for Postgres → replace helpers in `persistence/`
- To add a new bounty lifecycle action → add a route + update the data model

**Stack:** Node.js, Express, TypeScript, Zod for validation.

---

### Smart contract changes

**Location:** `contracts/src/lib.rs`

The Soroban contract implements the escrow lifecycle:

| Function         | What it does                                |
| ---------------- | ------------------------------------------- |
| `create_bounty`  | Locks funds and stores bounty metadata      |
| `reserve_bounty` | Assigns a contributor                       |
| `submit_bounty`  | Records a PR submission link                |
| `release_bounty` | Transfers escrowed funds to the contributor |
| `refund_bounty`  | Returns funds to the maintainer             |
| `get_bounty`     | Reads current bounty state                  |

**Build the contract:**

```bash
cd contracts
stellar contract build
```

**Run contract tests:**

```bash
cd contracts
cargo test
```

---

### Documentation changes

**Location:** `docs/issues/` and root `.md` files

Issue drafts in `docs/issues/` are meant to be opened as real GitHub issues. If you spot something missing or outdated, updating these files is a great first contribution.

---

## 8. Understanding the API

Base URLs:

- Direct: `http://localhost:3001`
- Via frontend proxy: `/api` (use this in frontend code)

| Method | Path                        | Description                                       |
| ------ | --------------------------- | ------------------------------------------------- |
| GET    | `/api/health`               | Liveness check                                    |
| GET    | `/api/bounties`             | List all bounties                                 |
| POST   | `/api/bounties`             | Create a new bounty                               |
| POST   | `/api/bounties/:id/reserve` | Reserve a bounty as a contributor                 |
| POST   | `/api/bounties/:id/submit`  | Submit a PR link                                  |
| POST   | `/api/bounties/:id/release` | Release payout to contributor                     |
| POST   | `/api/bounties/:id/refund`  | Refund escrow to maintainer                       |
| GET    | `/api/open-issues`          | List contribution-ready issues surfaced in the UI |

You can test endpoints with `curl` or any HTTP client (Insomnia, Postman, Thunder Client):

```bash
# Create a bounty
curl -X POST http://localhost:3001/api/bounties \
  -H "Content-Type: application/json" \
  -d '{"issueUrl": "https://github.com/org/repo/issues/1", "amount": 50, "token": "XLM"}'
```

---

## 9. Testing Your Changes

### Frontend

```bash
cd frontend
npm run lint         # TypeScript + ESLint checks
npm run build        # Catches type errors at compile time
```

> Automated component tests are a tracked contribution opportunity — see the open issues.

### Backend

```bash
cd backend
npm run lint
npm run build
```

Manually test API routes using curl or an HTTP client while `dev:backend` is running.

### Contract

```bash
cd contracts
cargo test           # Unit tests for contract logic
cargo clippy         # Rust linting
```

### End-to-end smoke test (manual)

1. Start both `dev:backend` and `dev:frontend`
2. Open `http://localhost:3000`
3. Create a bounty → reserve it → submit a PR link → release the payout
4. Confirm the status transitions are reflected in the UI and in `backend/data/bounties.json`

---

## 10. Picking Your First Issue

### If you are new to the codebase

Look for issues tagged **`good first issue`**. Good starting points include:

- **Documentation improvements** — clarifying the README, adding JSDoc comments, fixing typos
- **UI polish** — spacing, loading states, empty states, responsive layout tweaks
- **Validation improvements** — adding/tightening Zod schemas on the backend
- **Test coverage** — writing the first unit tests for backend routes or React components

These changes are self-contained, reviewable quickly, and help you learn the structure without needing to understand the full system.

---

### If you are comfortable with the stack

Look for issues tagged **`enhancement`** or **`help wanted`**. High-value areas include:

| Area                                        | What's involved                                                                                     |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| **Wallet-authenticated maintainer actions** | Connect Freighter or Albedo wallet; gate `release` and `refund` actions behind a signed transaction |
| **GitHub webhook sync**                     | Listen for PR merge/close events and automatically update bounty status                             |
| **Soroban event indexer**                   | Read emitted contract events and sync on-chain state back to the backend                            |
| **Postgres persistence**                    | Replace `backend/data/bounties.json` with a proper database layer                                   |
| **CI and integration tests**                | GitHub Actions workflow covering lint, build, and API integration tests                             |

---

### How to claim an issue

1. Comment on the issue with a brief description of your approach
2. Wait for a maintainer to assign it to you (usually within 24–48 hours)
3. Create a branch: `git checkout -b feat/your-issue-description`
4. Build, test locally, then open a PR

---

## 11. Issue Types Explained

| Label              | Meaning                                          |
| ------------------ | ------------------------------------------------ |
| `good first issue` | Small, well-scoped, minimal context needed       |
| `enhancement`      | New feature or meaningful improvement            |
| `help wanted`      | Maintainer wants community input; may be complex |
| `bug`              | Something is broken and needs a fix              |
| `documentation`    | Docs-only change, no code required               |
| `contract`         | Touches the Soroban Rust contract                |
| `backend`          | Touches the Express API                          |
| `frontend`         | Touches the React dashboard                      |

Issue drafts ready to be opened live in [`docs/issues/`](./docs/issues/). If you open one, apply the labels listed in the draft.

---

## 12. Submitting a Pull Request

1. **Branch from `main`** — keep your branch focused on one issue
2. **Write a clear PR title** — e.g. `feat: add wallet authentication for release action`
3. **Fill in the PR template** — describe what changed, why, and how to test it
4. **Link the issue** — include `Closes #<issue-number>` in the PR description
5. **Keep diffs small** — reviewers appreciate focused, reviewable changes
6. **Respond to review comments** — push follow-up commits to the same branch

**Before you commit:** Check out [CONTRIBUTING.md](./CONTRIBUTING.md) for our commit message style guide (conventional commits), PR checklist, and development best practices.

---

## 13. Architecture & Deployment Links

- **Live demo:** https://stellar-bounty-board-taupe.vercel.app
- **Stellar Developer Docs:** https://developers.stellar.org/docs
- **Soroban Smart Contracts:** https://developers.stellar.org/docs/build/smart-contracts/overview
- **Freighter Wallet (for local testing):** https://www.freighter.app
- **Stellar Testnet Friendbot (free test XLM):** https://friendbot.stellar.org

---

## 14. Getting Help

- **Open a Discussion** on GitHub if you are unsure about an approach before coding
- **Comment on the issue** you are working on if you get stuck
- **Check `docs/issues/`** for context on planned features — these drafts often contain useful background

We want contributing here to feel approachable. If this guide is missing something that tripped you up, a PR to improve it is one of the most valuable contributions you can make.

Happy building! 🚀

---

## 15. Stellar & Soroban Glossary

Not familiar with blockchain or the Stellar ecosystem? This section defines every Stellar and Soroban term used in these docs. Terms are ordered from foundational network concepts through to project-specific vocabulary so you can read straight through or jump to any entry.

For the complete official reference, see the [Stellar Glossary](https://developers.stellar.org/docs/learn/glossary).

---

### Network & Infrastructure

#### Stellar

The open-source, decentralised payment network this project builds on. Stellar is designed for fast, low-cost token transfers and supports smart contracts through the Soroban execution environment. Consensus is reached not by mining but by a federated agreement protocol called SCP.

→ [Stellar Consensus Protocol (SCP)](https://developers.stellar.org/docs/learn/fundamentals/stellar-consensus-protocol)

---

#### Ledger

A single "block" in the Stellar blockchain. Every ~5 seconds the network closes a new ledger that records all the transactions processed in that round. Ledger entries store account balances, contract state, and other on-chain data. When contract code or a bounty's state changes, the update is written to the current ledger.

→ [Ledgers](https://developers.stellar.org/docs/learn/fundamentals/stellar-data-structures/ledgers)

---

#### Testnet

A public Stellar network that mirrors the behaviour of the main network (Mainnet) but uses play-money tokens with no real-world value. You will run this project against Testnet during development — it is safe to experiment, make mistakes, and reset balances here without any financial risk.

→ [Networks overview](https://developers.stellar.org/docs/networks)

---

#### Futurenet

A pre-release Stellar network used to test features before they reach Testnet or Mainnet. Earlier versions of this project targeted Futurenet when Soroban was still maturing. You will normally use Testnet unless a contributor issue explicitly mentions Futurenet.

→ [Networks overview](https://developers.stellar.org/docs/networks)

---

#### Horizon

The public REST API for the Stellar network. Horizon lets you query account balances, transaction history, and ledger data without running a full node. The backend's event-indexer worker consults Horizon (and/or the Soroban RPC) to stay in sync with on-chain state.

→ [Horizon API](https://developers.stellar.org/docs/data/apis/horizon)

---

#### Soroban RPC (stellar-rpc)

A JSON-RPC endpoint for interacting with Soroban smart contracts specifically. While Horizon covers general Stellar data, the Soroban RPC handles contract simulation, submission, and event streaming. The backend connects to a Soroban RPC URL (set via the `SOROBAN_RPC_URL` environment variable) to call the escrow contract.

→ [Stellar RPC](https://developers.stellar.org/docs/data/apis/rpc)

---

#### Stellar CLI (`stellar`)

The official command-line tool for building, testing, deploying, and invoking Soroban contracts. The onboarding prerequisites ask you to install it; scripts like `npm run gen:bindings` call it internally to generate TypeScript types from the compiled contract.

→ [Stellar developer tools](https://developers.stellar.org/docs/tools) · [stellar-cli on GitHub](https://github.com/stellar/stellar-cli)

---

### Accounts, Tokens & Addresses

#### XLM / Lumens

The native token of the Stellar network. XLM is used to pay transaction fees and to satisfy the minimum base reserve that every account must hold to exist on the ledger. Bounty amounts in this project are denominated in XLM (or any Stellar asset) and held in escrow until a payout is released or refunded.

→ [Lumens (XLM)](https://developers.stellar.org/docs/learn/fundamentals/lumens)

---

#### Account

A record on the Stellar ledger identified by a public key (StrKey). Accounts hold balances, issue assets, and sign transactions. In this project the maintainer account funds the escrow, and the contributor account receives the payout.

→ [Accounts](https://developers.stellar.org/docs/learn/fundamentals/stellar-data-structures/accounts)

---

#### StrKey

The human-readable format Stellar uses for public keys, secret keys, and contract addresses. A public key looks like `GABC...XYZ` (starts with `G`); a contract ID looks like `CABC...XYZ` (starts with `C`). You will see StrKey addresses throughout the codebase and environment variables such as `MAINTAINER_PUBLIC_KEY` and `ARBITER_ADDRESS`.

→ [Account ID / StrKey](https://developers.stellar.org/docs/learn/fundamentals/stellar-data-structures/accounts#account-id-strkey)

---

#### Asset

Any token that can be transferred on Stellar. Assets are identified by a code (e.g., `XLM`, `USDC`) and an issuer address. The bounty contract can hold any Stellar asset in escrow, not just XLM.

→ [Assets](https://developers.stellar.org/docs/learn/fundamentals/stellar-data-structures/assets)

---

#### Transaction

A signed envelope that groups one or more operations (e.g., "pay 50 XLM to this address") and is submitted to the network atomically. Transactions expire after a defined time — if you see a "transaction expired" error, you need to build and sign a fresh one.

→ [Transactions](https://developers.stellar.org/docs/learn/fundamentals/transactions)

---

#### SEP (Stellar Ecosystem Proposal)

Standardised, community-authored specifications for how Stellar applications should behave interoperably. Think of SEPs as RFCs for the Stellar ecosystem. SEP-10 (Stellar Web Authentication) is particularly relevant here: it is the standard used to prove wallet ownership without exposing a secret key, and it underlies the planned wallet-authenticated maintainer actions feature.

→ [Stellar Ecosystem Proposals](https://github.com/stellar/stellar-protocol) · [SEP-10 (Stellar Web Authentication)](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0010.md)

---

#### XDR (External Data Representation)

The binary serialisation format Stellar uses to encode transactions, ledger entries, and contract values on the wire. You will encounter XDR if you inspect raw transaction envelopes or debug low-level RPC responses. The Stellar SDK encodes/decodes XDR for you automatically in most cases.

→ [XDR in the Stellar glossary](https://developers.stellar.org/docs/learn/glossary#xdr)

---

### Smart Contracts & Soroban

#### Soroban

Stellar's smart-contract execution platform, introduced in Protocol 20. Soroban contracts are written in Rust, compiled to WebAssembly (WASM), and deployed to the Stellar ledger. The `contracts/` directory in this repository contains the Soroban escrow contract.

→ [Soroban smart contracts overview](https://developers.stellar.org/docs/build/smart-contracts/overview) · [Contract development fundamentals](https://developers.stellar.org/docs/learn/fundamentals/contract-development)

---

#### WASM (WebAssembly)

The portable binary instruction format that Soroban contracts are compiled to before deployment. Running `stellar contract build` (or `cargo build --target wasm32-unknown-unknown`) produces a `.wasm` file that is uploaded to the ledger. The `npm run gen:bindings` script reads the WASM binary to generate TypeScript bindings for the frontend.

→ [WASM in the Stellar glossary](https://developers.stellar.org/docs/learn/glossary#wasm)

---

#### Contract ID

The on-chain StrKey address (`C...`) that uniquely identifies a deployed Soroban contract. Set the `SOROBAN_CONTRACT_ID` environment variable to point the backend at your deployed instance. Each deployment to a different network produces a different contract ID.

→ [Contract interactions](https://developers.stellar.org/docs/learn/fundamentals/contract-development/contract-interactions)

---

#### SAC (Stellar Asset Contract)

A pre-deployed Soroban contract that wraps any native Stellar asset (including XLM) and exposes it as a smart-contract token. Rather than implementing a custom token from scratch, the bounty contract uses the SAC to move XLM in and out of escrow atomically on-chain.

→ [Stellar Asset Contract](https://developers.stellar.org/docs/tokens/stellar-asset-contract)

---

#### Contract Storage

Key-value pairs written to the ledger by a Soroban contract and persisted between invocations. Soroban offers three storage tiers — `Persistent`, `Temporary`, and `Instance` — with different retention and fee characteristics. The bounty contract uses persistent storage to keep bounty records alive for the duration of the escrow lifecycle.

→ [Contract storage](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage)

---

### Project-Specific Terms

#### Escrow

The pattern by which the bounty contract holds funds on behalf of two parties (maintainer and contributor) until a condition is met. When a maintainer creates a bounty, XLM is transferred from their account into the contract. The contract releases those funds only when the maintainer calls `release_bounty` (paying the contributor) or `refund_bounty` (returning to the maintainer). Neither party can unilaterally move the funds while they are held in escrow.

→ [Soroban smart contracts overview](https://developers.stellar.org/docs/build/smart-contracts/overview)

---

#### Freighter

A browser extension wallet for the Stellar network — the Chrome/Firefox equivalent of MetaMask for Stellar. New contributors need Freighter to sign transactions locally during development and to test the wallet-authenticated maintainer actions that are on the roadmap.

→ [Freighter wallet](https://freighter.app/)

---

#### Friendbot

A testnet faucet that funds a Stellar account with free testnet XLM on demand. Call `https://friendbot.stellar.org/?addr=YOUR_PUBLIC_KEY` to get tokens for local testing. Friendbot only works on Testnet and Futurenet, never on Mainnet.

→ [Friendbot in the Stellar glossary](https://developers.stellar.org/docs/learn/glossary#friendbot)

---

> **Further reading:** The [Stellar Developer Docs](https://developers.stellar.org/docs) cover everything above in much greater depth. The [Stellar Glossary](https://developers.stellar.org/docs/learn/glossary) is a useful quick-reference for any term not listed here.
