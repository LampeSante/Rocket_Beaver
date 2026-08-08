# Rocket Beaver ($RBVR)
# Mainnet Deployment Runbook

## Purpose

Operational sequence for deploying the validated RBVR protocol to Solana Mainnet.

Reference Release:

rbvr-devnet-v1.0.0-immutable


---

# Phase 1 — Pre Deployment

Verify:

- [ ] Correct repository commit checked out
- [ ] Binary hash verified
- [ ] Deployment wallet confirmed
- [ ] Mainnet RPC configured
- [ ] SOL balance confirmed


---

# Phase 2 — Program Deployment

Deploy:

- [ ] Deploy validated treasury_router.so
- [ ] Record Program ID
- [ ] Record ProgramData address
- [ ] Record deployment transaction


---

# Phase 3 — Verification

Run:

- [ ] Program verification script
- [ ] Confirm binary details
- [ ] Confirm expected addresses
- [ ] Confirm initial authority


---

# Phase 4 — Economic Verification

Confirm:

- [ ] Treasury initialized
- [ ] Allocation model active
- [ ] Founder controls active
- [ ] Destination firewalls active


---

# Phase 5 — Immutability Lock

Execute:

- [ ] Remove upgrade authority
- [ ] Verify authority = none
- [ ] Record transaction signature


---

# Phase 6 — Final Verification

Confirm:

- [ ] Program immutable
- [ ] Treasury state correct
- [ ] Documentation updated
- [ ] Release published


---

# Deployment Completion

Final state:

DEPLOYED → VERIFIED → IMMUTABLE


