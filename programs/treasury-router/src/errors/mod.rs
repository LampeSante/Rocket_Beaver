use anchor_lang::prelude::*;

#[error_code]
pub enum TreasuryRouterError {
    #[msg("The supplied authority is not authorized.")]
    Unauthorized,

    #[msg("The protocol is currently paused.")]
    ProtocolPaused,

    #[msg("The treasury module has already been initialized.")]
    TreasuryAlreadyInitialized,

    #[msg("The protocol configuration has already been initialized.")]
    ProtocolConfigAlreadyInitialized,

    #[msg("The settlement mint cannot be the default public key.")]
    InvalidSettlementMint,

    #[msg("The supplied settlement-token vault is invalid.")]
    InvalidSettlementVault,

    #[msg("The settlement-token deposit amount must be greater than zero.")]
    InvalidDepositAmount,

    #[msg("There are no unprocessed settlement-token fees in the treasury vault.")]
    NoUnprocessedFees,

    #[msg("An arithmetic overflow occurred.")]
    ArithmeticOverflow,

    #[msg("The Beavernomics allocation configuration is invalid.")]
    InvalidAllocationConfiguration,

    #[msg("The founder module has already been initialized.")]
    FounderAlreadyInitialized,

    #[msg("The founder recipient cannot be the default public key.")]
    InvalidFounderRecipient,

    #[msg("The founder accounting-period cap must be greater than zero.")]
    InvalidFounderPeriodCap,

    #[msg("The founder accounting-period duration must be greater than zero.")]
    InvalidFounderPeriodDuration,

    #[msg("The company module has already been initialized.")]
    CompanyAlreadyInitialized,

    #[msg("The company recipient cannot be the default public key.")]
    InvalidCompanyRecipient,

    #[msg("The company accounting-period cap must be greater than zero.")]
    InvalidCompanyPeriodCap,

    #[msg("The company accounting-period duration must be greater than zero.")]
    InvalidCompanyPeriodDuration,

    #[msg("The requested release amount must be greater than zero.")]
    InvalidReleaseAmount,

    #[msg("Buyback execution is currently paused.")]
    BuybacksPaused,

    #[msg("The Dam is closed and no treasury release is permitted.")]
    DamClosed,

    #[msg("The requested release exceeds the pending treasury balance.")]
    InsufficientPendingBalance,

    #[msg("The requested release exceeds the current Dam release limit.")]
    ReleaseLimitExceeded,

    #[msg("The Dam release rate is invalid.")]
    InvalidDamReleaseRate,

    #[msg("The treasury is not linked to the supplied protocol.")]
    InvalidTreasuryProtocol,

    #[msg("The protocol treasury module has not been initialized.")]
    TreasuryNotInitialized,

    #[msg("A treasury accounting invariant was violated.")]
    AccountingInvariantViolation,

    #[msg("Treasury accounting must be fully settled before funds can be released.")]
    AccountingNotSettled,
    #[msg("Invalid settlement-token execution destination")]
    InvalidExecutionDestination,

    #[msg("The Integrity Firewall rejected the supplied account architecture.")]
    IntegrityFirewallViolation,

    #[msg("The Reserve Policy has already been initialized.")]
    ReservePolicyAlreadyInitialized,

    #[msg("The Reserve Policy minimum floor must be greater than zero.")]
    InvalidReserveMinimumFloor,

    #[msg("The Reserve Policy liquidity-floor rate is invalid.")]
    InvalidReserveLiquidityFloorRate,

    #[msg("The Reserve Policy surplus-deployment rate is invalid.")]
    InvalidReserveDeploymentRate,

    #[msg("The Reserve Policy deployment cooldown must be greater than zero.")]
    InvalidReserveDeploymentCooldown,

    #[msg("The requested Reserve deployment amount must be greater than zero.")]
    InvalidReserveDeploymentAmount,

    #[msg("The Reserve deployment cooldown is still active.")]
    ReserveDeploymentCooldownActive,

    #[msg("The supplied Reserve Policy is not linked to this protocol and treasury.")]
    InvalidReservePolicyLinkage,

    #[msg("The Reserve Vault does not currently contain deployable surplus.")]
    NoDeployableReserveSurplus,

    #[msg("The Reserve deployment calculation failed closed.")]
    ReserveDeploymentEvaluationFailed,

    #[msg("The Spillway destination is invalid.")]
    InvalidSpillwayDestination,

    #[msg("The Spillway would breach the protected Reserve floor.")]
    ReserveFloorViolation,
}
