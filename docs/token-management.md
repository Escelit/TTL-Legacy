# Token Management Features

This document describes the token management features implemented in TTL-Legacy, including token conversion, staking, yield distribution, lending, collateral, hedging, and rebalancing.

## Overview

TTL-Legacy now supports advanced token management capabilities that allow vault owners to:

1. **Convert tokens** before release (Issue #581)
2. **Validate token whitelisting** in batch operations (Issue #582)
3. **Stake tokens** for yield while locked (Issue #583)
4. **Distribute or reinvest yield** (Issue #584)
5. **Lend vault tokens** for interest income (Issue #585)
6. **Use tokens as collateral** for loans (Issue #586)
7. **Hedge token price risk** using derivatives (Issue #587)
8. **Rebalance multi-token portfolios** based on target weights (Issue #588)

## Issue #581: Token Conversion

### Purpose

Allow vault owners to convert their vault tokens to different tokens before the vault is released to the beneficiary. This is useful for:

- Converting to stablecoins before release
- Exchanging to preferred tokens
- Hedging against price volatility

### API

#### Enable Token Conversion

```rust
pub fn enable_token_conversion(
    env: Env,
    vault_id: u64,
    from_token: Address,
    to_token: Address,
    conversion_rate: i128,
)
```

**Parameters:**
- `vault_id`: The vault ID
- `from_token`: Source token address (must be whitelisted)
- `to_token`: Target token address (must be whitelisted)
- `conversion_rate`: Conversion rate in basis points (10000 = 1:1)

**Requirements:**
- Caller must be the vault owner
- Both tokens must be whitelisted
- Conversion rate must be positive

**Events:**
- `TOKEN_CONVERSION_TOPIC`: Emitted when conversion is enabled

#### Get Token Conversion

```rust
pub fn get_token_conversion(env: Env, vault_id: u64) -> Option<TokenConversion>
```

Returns the token conversion configuration for a vault, or `None` if not configured.

### Example Usage

```rust
// Enable conversion from XLM to USDC at 1:1 rate
client.enable_token_conversion(
    &vault_id,
    &xlm_token,
    &usdc_token,
    &10000i128, // 1:1 rate
);

// Retrieve conversion config
if let Some(conversion) = client.get_token_conversion(&vault_id) {
    println!("Converting {} to {} at rate {}", 
        conversion.from_token, 
        conversion.to_token, 
        conversion.conversion_rate);
}
```

## Wrapped Token Support for Cross-Chain Compatibility

### Purpose

Allow vault owners to use wrapped tokens that represent a canonical token from another chain or bridge.
Wrapped tokens are accepted whenever their registered canonical token is whitelisted.

### API

#### Register Wrapped Token

```rust
pub fn register_wrapped_token(
    env: Env,
    wrapped_token_address: Address,
    canonical_token_address: Address,
)
```

### Example Usage

```rust
client.register_wrapped_token(&wrapped_token, &xlm_token);
```

Wrapped token registration makes it possible to create vaults and deposit with the wrapped asset while still enforcing the canonical token whitelist.

## Issue #582: Token Whitelisting Validation

### Purpose

Ensure that only whitelisted tokens can be deposited into vaults through batch operations. This prevents accidental or malicious use of non-approved tokens.

### Implementation

The `batch_deposit` function now validates that each vault's token is whitelisted before processing deposits:

```rust
pub fn batch_deposit(env: Env, from: Address, deposits: Vec<(u64, i128)>) {
    // ... validation ...
    
    // Issue #582: Validate token whitelist
    Self::assert_token_whitelisted(&env, &vault.token_address);
    
    // ... process deposit ...
    
    // Emit token whitelist validation event
    env.events().publish(
        (TOKEN_WHITELIST_VALIDATED_TOPIC, vault_id),
        (&vault.token_address, amount),
    );
}
```

### Validation Rules

1. Default XLM token is always whitelisted
2. Custom tokens must be explicitly whitelisted by admin
3. Validation occurs for each vault in the batch before any transfers
4. If any vault uses a non-whitelisted token, the entire batch is rejected

### Events

- `TOKEN_WHITELIST_VALIDATED_TOPIC`: Emitted for each successfully validated deposit

## Issue #583: Token Staking

### Purpose

Allow vault owners to stake their vault tokens in external staking pools to earn yield while the vault is locked. This enables passive income generation during the vault's active period.

### API

#### Enable Token Staking

```rust
pub fn enable_token_staking(
    env: Env,
    vault_id: u64,
    staking_pool: Address,
    annual_yield_bps: u32,
)
```

**Parameters:**
- `vault_id`: The vault ID
- `staking_pool`: Address of the staking pool contract
- `annual_yield_bps`: Annual yield in basis points (e.g., 500 = 5%)

**Requirements:**
- Caller must be the vault owner
- Annual yield must be between 0 and 10000 basis points

**Events:**
- `TOKEN_STAKING_TOPIC`: Emitted when staking is enabled

#### Disable Token Staking

```rust
pub fn disable_token_staking(env: Env, vault_id: u64)
```

Disables staking for a vault. The vault owner can call this to stop earning yield.

**Events:**
- `TOKEN_UNSTAKING_TOPIC`: Emitted when staking is disabled

#### Get Token Staking

```rust
pub fn get_token_staking(env: Env, vault_id: u64) -> Option<TokenStaking>
```

Returns the staking configuration for a vault.

### Example Usage

```rust
// Enable staking with 5% annual yield
client.enable_token_staking(
    &vault_id,
    &staking_pool_address,
    &500u32, // 5% APY
);

// Check staking status
if let Some(staking) = client.get_token_staking(&vault_id) {
    println!("Staking {} tokens at {}% APY",
        staking.staked_amount,
        staking.annual_yield_bps as f64 / 100.0);
}

// Disable staking
client.disable_token_staking(&vault_id);
```

## Issue #584: Token Yield Distribution

### Purpose

Configure how staking yield is distributed or reinvested. Vault owners can choose to:

1. **Distribute to Beneficiary**: Send all yield to the beneficiary
2. **Reinvest**: Automatically reinvest yield back into the vault
3. **Split**: Distribute a percentage to beneficiary and reinvest the rest

### API

#### Set Yield Distribution

```rust
pub fn set_yield_distribution(
    env: Env,
    vault_id: u64,
    mode: YieldDistributionMode,
)
```

**Parameters:**
- `vault_id`: The vault ID
- `mode`: The distribution mode (see below)

**Yield Distribution Modes:**

```rust
pub enum YieldDistributionMode {
    /// Distribute all yield to beneficiary
    DistributeToBeneficiary,
    
    /// Reinvest all yield back into vault
    Reinvest,
    
    /// Split yield: beneficiary_bps to beneficiary, rest reinvested
    Split(u32), // basis points for beneficiary
}
```

**Requirements:**
- Caller must be the vault owner
- Vault must have staking enabled

**Events:**
- `YIELD_DISTRIBUTED_TOPIC`: Emitted when yield is distributed

#### Get Yield Distribution

```rust
pub fn get_yield_distribution(env: Env, vault_id: u64) -> Option<YieldDistributionConfig>
```

Returns the yield distribution configuration for a vault.

#### Distribute Yield

```rust
pub fn distribute_yield(env: Env, vault_id: u64)
```

Calculates accumulated yield and distributes it according to the configured mode.

**Yield Calculation:**

```
yield = (staked_amount × annual_yield_bps × time_elapsed) / (10000 × 365 × 86400)
```

**Events:**
- `YIELD_DISTRIBUTED_TOPIC`: Emitted when yield is sent to beneficiary
- `YIELD_REINVESTED_TOPIC`: Emitted when yield is reinvested

### Example Usage

```rust
// Distribute all yield to beneficiary
client.set_yield_distribution(
    &vault_id,
    &YieldDistributionMode::DistributeToBeneficiary,
);

// Or reinvest all yield
client.set_yield_distribution(
    &vault_id,
    &YieldDistributionMode::Reinvest,
);

// Or split 70% to beneficiary, 30% reinvest
client.set_yield_distribution(
    &vault_id,
    &YieldDistributionMode::Split(7000u32),
);

// Distribute accumulated yield
client.distribute_yield(&vault_id);

// Check distribution stats
if let Some(config) = client.get_yield_distribution(&vault_id) {
    println!("Total distributed: {}", config.total_distributed);
    println!("Total reinvested: {}", config.total_reinvested);
}
```

## Integration Example

Here's a complete example showing how to use all token management features together:

```rust
// 1. Create a vault
let vault_id = client.create_vault(&owner, &beneficiary, &86400u64, &None);

// 2. Deposit funds
client.deposit(&vault_id, &owner, &1_000_000i128);

// 3. Enable staking with 5% APY
client.enable_token_staking(&vault_id, &staking_pool, &500u32);

// 4. Set yield distribution (70% to beneficiary, 30% reinvest)
client.set_yield_distribution(
    &vault_id,
    &YieldDistributionMode::Split(7000u32),
);

// 5. Enable token conversion (optional)
client.enable_token_conversion(
    &vault_id,
    &xlm_token,
    &usdc_token,
    &10000i128,
);

// 6. After some time, distribute yield
client.distribute_yield(&vault_id);

// 7. Check final state
let config = client.get_yield_distribution(&vault_id).unwrap();
println!("Distributed to beneficiary: {}", config.total_distributed);
println!("Reinvested: {}", config.total_reinvested);
```

## Issue #585: Token Lending

### Purpose

Allow vault owners to lend vault tokens to a borrower and earn interest income.

### API

#### Enable Token Lending

```rust
pub fn enable_token_lending(
    env: Env,
    vault_id: u64,
    caller: Address,
    borrower: Address,
    amount: i128,
    interest_rate_bps: u32,
    duration_seconds: u64,
) -> Result<(), ContractError>
```

**Parameters:**
- `vault_id`: The vault ID
- `caller`: Must be the vault owner
- `borrower`: Address of the borrower
- `amount`: Amount to lend (must be ≤ vault balance)
- `interest_rate_bps`: Annual interest rate in basis points (e.g., 500 = 5%)
- `duration_seconds`: Loan duration in seconds

**Events:**
- `TOKEN_LENDING_TOPIC`: Emitted when lending is enabled

#### Repay Token Loan

```rust
pub fn repay_token_loan(env: Env, vault_id: u64, caller: Address) -> Result<i128, ContractError>
```

Returns the accrued interest earned.

**Events:**
- `TOKEN_LEND_REPAY_TOPIC`: Emitted on repayment

#### Get Token Lending

```rust
pub fn get_token_lending(env: Env, vault_id: u64) -> Option<TokenLending>
```

## Issue #586: Token Collateral

### Purpose

Allow vault owners to use vault tokens as collateral for an external loan.

### API

#### Set Token Collateral

```rust
pub fn set_token_collateral(
    env: Env,
    vault_id: u64,
    caller: Address,
    collateral_amount: i128,
    loan_amount: i128,
    collateral_ratio_bps: u32,
) -> Result<(), ContractError>
```

**Parameters:**
- `collateral_ratio_bps`: Required collateral ratio ≥ 10000 (100%)

**Events:**
- `TOKEN_COLLATERAL_TOPIC`: Emitted when collateral is set

#### Release Token Collateral

```rust
pub fn release_token_collateral(env: Env, vault_id: u64, caller: Address) -> Result<(), ContractError>
```

**Events:**
- `TOKEN_COLLAT_RLSD_TOPIC`: Emitted when collateral is released

#### Get Token Collateral

```rust
pub fn get_token_collateral(env: Env, vault_id: u64) -> Option<TokenCollateral>
```

## Issue #587: Token Hedging

### Purpose

Allow vault owners to hedge token price risk using a derivative position.

### API

#### Enable Token Hedge

```rust
pub fn enable_token_hedge(
    env: Env,
    vault_id: u64,
    caller: Address,
    hedge_token: Address,
    notional_amount: i128,
    strike_price_bps: u32,
    expiry: u64,
) -> Result<(), ContractError>
```

**Events:**
- `TOKEN_HEDGE_TOPIC`: Emitted when hedge is enabled

#### Close Token Hedge

```rust
pub fn close_token_hedge(env: Env, vault_id: u64, caller: Address) -> Result<(), ContractError>
```

**Events:**
- `TOKEN_HEDGE_CLOSE_TOPIC`: Emitted when hedge is closed

#### Get Token Hedge

```rust
pub fn get_token_hedge(env: Env, vault_id: u64) -> Option<TokenHedge>
```

## Issue #588: Token Rebalancing

### Purpose

Automatically rebalance a multi-token vault portfolio based on configured target weights.

### API

#### Set Token Rebalance

```rust
pub fn set_token_rebalance(
    env: Env,
    vault_id: u64,
    caller: Address,
    target_weights: Vec<TokenWeight>,
    rebalance_threshold_bps: u32,
) -> Result<(), ContractError>
```

**Parameters:**
- `target_weights`: Per-token allocations; `target_bps` values must sum to 10000
- `rebalance_threshold_bps`: Drift tolerance before triggering a rebalance (e.g., 500 = 5%)

**Events:**
- `TOKEN_REBALANCE_TOPIC`: Emitted when rebalance config is set

#### Trigger Rebalance

```rust
pub fn trigger_rebalance(env: Env, vault_id: u64) -> Result<(), ContractError>
```

**Events:**
- `TOKEN_REBALANCED_TOPIC`: Emitted on each rebalance

#### Get Token Rebalance

```rust
pub fn get_token_rebalance(env: Env, vault_id: u64) -> Option<TokenRebalanceConfig>
```

## Security Considerations

1. **Token Whitelisting**: Only whitelisted tokens can be used in vaults
2. **Owner Authorization**: Only vault owners can configure staking, lending, collateral, hedging, and rebalancing
3. **Yield Calculation**: Yield is calculated based on time elapsed and annual rate
4. **Atomic Operations**: Batch deposits validate all items before any transfers
5. **Event Tracking**: All operations emit events for on-chain audit trails
6. **Collateral Ratio**: Collateral ratio must be ≥ 100% to prevent under-collateralised loans
7. **Balance Checks**: Lending and collateral operations verify sufficient vault balance
