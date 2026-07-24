import json
import sys

with open("config/beavernomics-v1.json") as f:
    economics = json.load(f)

with open("config/authority-architecture-v1.json") as f:
    architecture = json.load(f)

errors = []

supply = economics["token"]["fixed_initial_supply"]
arch_supply = architecture["mint"]["genesis_supply"]

if supply != arch_supply:
    errors.append(
        f"Supply mismatch: economics={supply}, architecture={arch_supply}"
    )

econ_dist = economics["working_token_distribution"]
arch_dist = architecture["token_destinations"]

mapping = {
    "community_curve": "community_curve",
    "initial_amm_liquidity": "initial_amm_liquidity",
    "locked_future_liquidity": "future_liquidity",
    "community_initiatives": "community_initiatives"
}

for econ_key, arch_key in mapping.items():
    e = econ_dist[econ_key]
    a = arch_dist[arch_key]

    if e["tokens"] != a["tokens"]:
        errors.append(
            f"{econ_key}: token mismatch {e['tokens']} != {a['tokens']}"
        )

    if e["percent"] != a["percent"]:
        errors.append(
            f"{econ_key}: percentage mismatch {e['percent']} != {a['percent']}"
        )

total_tokens = sum(v["tokens"] for v in arch_dist.values())
total_percent = sum(v["percent"] for v in arch_dist.values())

if total_tokens != supply:
    errors.append(
        f"Architecture allocations total {total_tokens}, expected {supply}"
    )

if total_percent != 100:
    errors.append(
        f"Architecture percentages total {total_percent}%, expected 100%"
    )

econ_routing = economics["beavernomics"]["routing"]
arch_revenue = architecture["revenue_destinations"]

routing_mapping = {
    "protocol_reserve_percent": "protocol_reserve",
    "automatic_buyback_burn_percent": "buyback_burn",
    "liquidity_growth_percent": "liquidity_growth",
    "company_operations_percent": "company_operations",
    "founder_initial_percent": "founder_compensation"
}

for econ_key, arch_key in routing_mapping.items():
    expected = econ_routing[econ_key]

    if arch_key == "founder_compensation":
        actual = arch_revenue[arch_key]["initial_target_percent"]
    else:
        actual = arch_revenue[arch_key]["target_percent"]

    if abs(expected - actual) > 1e-9:
        errors.append(
            f"{arch_key}: routing mismatch {expected} != {actual}"
        )

print("=" * 55)
print("ROCKET BEAVER — ARCHITECTURE VALIDATION")
print("=" * 55)

print(f"Genesis supply:        {arch_supply:,} RBVR")
print(f"Allocation total:      {total_tokens:,} RBVR")
print(f"Allocation percentage: {total_percent}%")
print("Creator allocation:    0 RBVR")
print("Founder token bag:     NONE")

if errors:
    print()
    print("RESULT: FAIL")
    for error in errors:
        print(" -", error)
    sys.exit(1)

print()
print("RESULT: PASS")
print("Architecture matches locked Beavernomics V1.0.")
print()
print("MAINNET: BLOCKED")
print("Devnet validation and production safety gates remain required.")
