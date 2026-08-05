# Beavernomics

## Base allocation

| Bucket | Basis points | Percentage |
|---|---:|---:|
| Reserve | 3000 | 30% |
| Buyback/Burn | 2000 | 20% |
| Liquidity Growth | 2000 | 20% |
| Company/Operations | 2000 | 20% |
| Founder Compensation | 1000 | 10% |

## Company controls

Company allocation is capped per period, not for the life of the protocol. When the current-period cap is reached, excess Company allocation is redirected to Liquidity Growth.

## Founder controls

Founder compensation receives a fixed 10% of the protocol’s 1% fee. This is equivalent to 0.1% of eligible transaction volume before the annual cap is applied. The Founder rate does not decline or change with volume.

When the annual Founder cap is reached, all excess Founder allocation is redirected to Liquidity Growth.

## Conservation

For every processed amount:

```text
Reserve + Buyback/Burn + Liquidity + Company + Founder = processed fees
```

After cap handling:

```text
final liquidity = base liquidity + company overflow + founder overflow
```

No token unit may disappear or be created through accounting.

## Reserve policy

Reserve capital is retained to support protocol resilience. Surplus deployment is governed by the Reserve Policy and Spillway rather than caller discretion.

## Buyback execution

The accounting bucket is on-chain. Any external execution strategy—such as TWAP, price-impact limits, or oracle checks—must be documented and reviewed separately unless implemented on-chain.

## Disclaimer

Beavernomics specifies treasury routing. It does not guarantee price performance, profit, liquidity, or investment return.
