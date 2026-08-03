/// Root protocol configuration PDA.
pub const PROTOCOL_SEED: &[u8] = b"protocol";

/// On-chain Beavernomics configuration PDA.
pub const PROTOCOL_CONFIG_SEED: &[u8] = b"protocol-config";

/// Treasury accounting PDA.
pub const TREASURY_SEED: &[u8] = b"treasury";

/// Treasury SPL settlement-token vault PDA.
pub const TREASURY_VAULT_SEED: &[u8] = b"treasury-vault";

/// Protocol-controlled reserve settlement-token vault.
pub const RESERVE_VAULT_SEED: &[u8] = b"reserve-vault";

/// Founder module PDA seed.
pub const FOUNDER_STATE_SEED: &[u8] = b"founder-state";

/// Company module PDA seed.
pub const COMPANY_STATE_SEED: &[u8] = b"company-state";

/// Immutable execution routing configuration PDA.
pub const EXECUTION_CONFIG_SEED: &[u8] = b"execution-config";

/// Current on-chain state versions.
pub const PROTOCOL_VERSION: u16 = 1;
pub const PROTOCOL_CONFIG_VERSION: u16 = 1;
pub const TREASURY_VERSION: u16 = 1;
pub const FOUNDER_STATE_VERSION: u16 = 1;
pub const COMPANY_STATE_VERSION: u16 = 1;
pub const EXECUTION_CONFIG_VERSION: u16 = 1;

/// Initial Dam Level: Building.
pub const INITIAL_DAM_LEVEL: u8 = 0;

/// 10,000 basis points equals 100%.
pub const BPS_DENOMINATOR: u16 = 10_000;

/// Initial Beavernomics distribution of collected fee proceeds.
pub const INITIAL_RESERVE_BPS: u16 = 3_000;
pub const INITIAL_BUYBACK_BURN_BPS: u16 = 2_000;
pub const INITIAL_LIQUIDITY_BPS: u16 = 2_000;
pub const INITIAL_COMPANY_BPS: u16 = 2_000;
pub const INITIAL_FOUNDER_BPS: u16 = 1_000;

/// Initial Survival Waterfall stage.
pub const INITIAL_WATERFALL_STAGE: u8 = 0;

/// Immutable Reserve Policy PDA seed.
pub const RESERVE_POLICY_SEED: &[u8] = b"reserve-policy";

/// Initial Reserve Policy account layout version.
pub const RESERVE_POLICY_VERSION: u16 = 1;

/// Maximum valid basis-point value.
pub const RESERVE_POLICY_BPS_DENOMINATOR: u16 = 10_000;
