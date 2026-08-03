#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

echo "===== RT-001: ATTACK SURFACE INVENTORY ====="
python3 rbvr-red-team-rt001.py

echo
echo "===== REPORT SUMMARY ====="
sed -n '1,220p' red-team/RT-001-ATTACK-SURFACE-INVENTORY.md

echo
echo "===== CRITICAL/HIGH REGISTER ====="
awk -F',' '
NR == 1 || $3 == "CRITICAL" || $3 == "HIGH" {
    print
}
' red-team/RT-001-ATTACK-REGISTER.csv

echo
echo "===== CURRENT STATUS ====="
git status --short
