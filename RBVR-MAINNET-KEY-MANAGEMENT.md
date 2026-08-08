# Rocket Beaver ($RBVR)
# Mainnet Key Management Plan

## Purpose

Define wallet responsibilities before mainnet deployment.

The objective is to prevent operational key concentration and ensure deployment integrity.

---

# Deployment Wallet

Purpose:

- Deploy program binary
- Pay deployment costs
- Verify deployment

Requirements:

- Dedicated wallet
- Not used for treasury operations
- Private key secured separately


---

# Upgrade Authority

Initial State:

Required during deployment only.

After verification:

- Upgrade authority revoked
- Authority set to none
- Program becomes immutable


---

# Treasury Operations

Treasury destinations must remain:

- PDA controlled
- Program derived
- Not controlled by deployment wallet


---

# Verification Wallet

Purpose:

- Verify program state
- Verify addresses
- Verify authority status

No operational permissions.


---

# Security Requirements

- No shared private keys
- No personal wallet deployment
- Hardware wallet preferred
- Recovery procedures documented


---

# Final Deployment Rule

Deploy → Verify → Revoke Authority → Verify Again


Status:

READY FOR MAINNET KEY CONFIGURATION

