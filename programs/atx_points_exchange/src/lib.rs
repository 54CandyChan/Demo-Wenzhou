use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

declare_id!("Fg6PaFpoGXkYsidMpWxTWqkqkR3Rr1VQw7B7h2xq1dJ");

#[program]
pub mod atx_points_exchange {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        require_keys_neq!(ctx.accounts.owner.key(), Pubkey::default(), ExchangeError::InvalidAddress);
        require_keys_neq!(ctx.accounts.atx_mint.key(), Pubkey::default(), ExchangeError::InvalidAddress);

        let config = &mut ctx.accounts.config;
        config.owner = ctx.accounts.owner.key();
        config.is_paused = false;
        config.points_per_exchange = GlobalConfig::DEFAULT_POINTS_PER_EXCHANGE;
        config.atx_per_exchange = GlobalConfig::DEFAULT_ATX_PER_EXCHANGE;
        config.atx_mint = ctx.accounts.atx_mint.key();
        config.exchange_counter = 0;

        emit!(ConfigInitialized {
            owner: config.owner,
            atx_mint: config.atx_mint,
            points_per_exchange: config.points_per_exchange,
            atx_per_exchange: config.atx_per_exchange,
        });

        Ok(())
    }

    pub fn add_user_points(ctx: Context<UpdateUserPoints>, points: u64) -> Result<()> {
        require_owner(&ctx.accounts.config, &ctx.accounts.owner)?;
        require!(points > 0, ExchangeError::InvalidPointsAmount);

        let user_points = &mut ctx.accounts.user_points;
        initialize_user_points_if_needed(user_points, ctx.accounts.user.key())?;
        user_points.points = user_points
            .points
            .checked_add(points)
            .ok_or(ExchangeError::MathOverflow)?;

        emit!(PointsAdjusted {
            user: ctx.accounts.user.key(),
            operator: ctx.accounts.owner.key(),
            delta: points as i128,
            new_balance: user_points.points,
        });

        Ok(())
    }

    pub fn sub_user_points(ctx: Context<UpdateUserPoints>, points: u64) -> Result<()> {
        require_owner(&ctx.accounts.config, &ctx.accounts.owner)?;
        require!(points > 0, ExchangeError::InvalidPointsAmount);

        let user_points = &mut ctx.accounts.user_points;
        initialize_user_points_if_needed(user_points, ctx.accounts.user.key())?;
        require!(
            user_points.points >= points,
            ExchangeError::InsufficientPoints
        );

        user_points.points = user_points
            .points
            .checked_sub(points)
            .ok_or(ExchangeError::MathOverflow)?;

        emit!(PointsAdjusted {
            user: ctx.accounts.user.key(),
            operator: ctx.accounts.owner.key(),
            delta: -(points as i128),
            new_balance: user_points.points,
        });

        Ok(())
    }

    pub fn toggle_pause(ctx: Context<TogglePause>, is_paused: bool) -> Result<()> {
        require_owner(&ctx.accounts.config, &ctx.accounts.owner)?;
        ctx.accounts.config.is_paused = is_paused;

        emit!(PauseStatusChanged {
            operator: ctx.accounts.owner.key(),
            is_paused,
        });

        Ok(())
    }

    pub fn exchange_points(ctx: Context<ExchangePoints>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require_config_ready(config)?;
        require!(
            !config.is_paused,
            ExchangeError::ExchangePaused
        );
        require_keys_eq!(
            ctx.accounts.atx_mint.key(),
            config.atx_mint,
            ExchangeError::InvalidMint
        );

        let user_key = ctx.accounts.user.key();
        let user_points = &mut ctx.accounts.user_points;
        initialize_user_points_if_needed(user_points, user_key)?;
        require_keys_eq!(user_points.owner, user_key, ExchangeError::UnauthorizedUser);
        require!(
            user_points.points >= config.points_per_exchange,
            ExchangeError::InsufficientPoints
        );

        user_points.points = user_points
            .points
            .checked_sub(config.points_per_exchange)
            .ok_or(ExchangeError::MathOverflow)?;
        user_points.exchange_count = user_points
            .exchange_count
            .checked_add(1)
            .ok_or(ExchangeError::MathOverflow)?;
        user_points.last_exchange_time = Clock::get()?.unix_timestamp;

        let signer_seeds: &[&[&[u8]]] = &[&[
            VaultAuthority::SEED_PREFIX,
            config.key().as_ref(),
            &[ctx.bumps.vault_authority],
        ]];
        let transfer_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        };
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        token::transfer(transfer_ctx, config.atx_per_exchange)
            .map_err(|_| error!(ExchangeError::TransferFailed))?;

        let exchange_id = config.exchange_counter;
        config.exchange_counter = config
            .exchange_counter
            .checked_add(1)
            .ok_or(ExchangeError::MathOverflow)?;

        let record = &mut ctx.accounts.exchange_record;
        record.user = user_key;
        record.points_used = config.points_per_exchange;
        record.atx_received = config.atx_per_exchange;
        record.timestamp = user_points.last_exchange_time;
        record.exchange_id = exchange_id;

        emit!(PointsExchanged {
            exchange_id,
            user: user_key,
            points_used: record.points_used,
            atx_received: record.atx_received,
            timestamp: record.timestamp,
        });

        Ok(())
    }

    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>, amount: u64) -> Result<()> {
        require_owner(&ctx.accounts.config, &ctx.accounts.owner)?;
        require!(amount > 0, ExchangeError::InvalidTokenAmount);
        require_keys_eq!(
            ctx.accounts.atx_mint.key(),
            ctx.accounts.config.atx_mint,
            ExchangeError::InvalidMint
        );

        let signer_seeds: &[&[&[u8]]] = &[&[
            VaultAuthority::SEED_PREFIX,
            ctx.accounts.config.key().as_ref(),
            &[ctx.bumps.vault_authority],
        ]];
        let transfer_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.owner_token_account.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        };
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        token::transfer(transfer_ctx, amount).map_err(|_| error!(ExchangeError::TransferFailed))?;

        emit!(TokensWithdrawn {
            operator: ctx.accounts.owner.key(),
            amount,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        space = 8 + GlobalConfig::INIT_SPACE,
        seeds = [GlobalConfig::SEED_PREFIX],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,
    pub atx_mint: Account<'info, Mint>,
    #[account(
        seeds = [VaultAuthority::SEED_PREFIX, config.key().as_ref()],
        bump
    )]
    /// CHECK: PDA authority with no data. It only signs CPI calls for the vault token account.
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = atx_mint,
        associated_token::authority = vault_authority
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateUserPoints<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [GlobalConfig::SEED_PREFIX],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,
    /// CHECK: Only the public key is used to derive the user's PDA.
    pub user: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + UserPoints::INIT_SPACE,
        seeds = [UserPoints::SEED_PREFIX, user.key().as_ref()],
        bump
    )]
    pub user_points: Account<'info, UserPoints>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TogglePause<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [GlobalConfig::SEED_PREFIX],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,
}

#[derive(Accounts)]
pub struct ExchangePoints<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [GlobalConfig::SEED_PREFIX],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserPoints::INIT_SPACE,
        seeds = [UserPoints::SEED_PREFIX, user.key().as_ref()],
        bump
    )]
    pub user_points: Account<'info, UserPoints>,
    #[account(
        init,
        payer = user,
        space = 8 + ExchangeRecord::INIT_SPACE,
        seeds = [ExchangeRecord::SEED_PREFIX, &config.exchange_counter.to_le_bytes()],
        bump
    )]
    pub exchange_record: Account<'info, ExchangeRecord>,
    pub atx_mint: Account<'info, Mint>,
    #[account(
        seeds = [VaultAuthority::SEED_PREFIX, config.key().as_ref()],
        bump
    )]
    /// CHECK: PDA authority with no data. It only signs CPI calls for token transfers.
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = atx_mint,
        associated_token::authority = vault_authority
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = atx_mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [GlobalConfig::SEED_PREFIX],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,
    pub atx_mint: Account<'info, Mint>,
    #[account(
        seeds = [VaultAuthority::SEED_PREFIX, config.key().as_ref()],
        bump
    )]
    /// CHECK: PDA authority with no data. It only signs CPI calls for token transfers.
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = atx_mint,
        associated_token::authority = vault_authority
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = atx_mint,
        associated_token::authority = owner
    )]
    pub owner_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
#[derive(InitSpace)]
pub struct UserPoints {
    pub owner: Pubkey,
    pub points: u64,
    pub exchange_count: u64,
    pub last_exchange_time: i64,
}

impl UserPoints {
    pub const SEED_PREFIX: &'static [u8] = b"user-points";
}

#[account]
#[derive(InitSpace)]
pub struct ExchangeRecord {
    pub user: Pubkey,
    pub points_used: u64,
    pub atx_received: u64,
    pub timestamp: i64,
    pub exchange_id: u64,
}

impl ExchangeRecord {
    pub const SEED_PREFIX: &'static [u8] = b"exchange-record";
}

#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub owner: Pubkey,
    pub is_paused: bool,
    pub points_per_exchange: u64,
    pub atx_per_exchange: u64,
    pub atx_mint: Pubkey,
    pub exchange_counter: u64,
}

impl GlobalConfig {
    pub const SEED_PREFIX: &'static [u8] = b"global-config";
    pub const DEFAULT_POINTS_PER_EXCHANGE: u64 = 1_000;
    pub const DEFAULT_ATX_PER_EXCHANGE: u64 = 1_000_000;
}

pub struct VaultAuthority;

impl VaultAuthority {
    pub const SEED_PREFIX: &'static [u8] = b"vault-authority";
}

#[event]
pub struct ConfigInitialized {
    pub owner: Pubkey,
    pub atx_mint: Pubkey,
    pub points_per_exchange: u64,
    pub atx_per_exchange: u64,
}

#[event]
pub struct PointsAdjusted {
    pub user: Pubkey,
    pub operator: Pubkey,
    pub delta: i128,
    pub new_balance: u64,
}

#[event]
pub struct PauseStatusChanged {
    pub operator: Pubkey,
    pub is_paused: bool,
}

#[event]
pub struct PointsExchanged {
    pub exchange_id: u64,
    pub user: Pubkey,
    pub points_used: u64,
    pub atx_received: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokensWithdrawn {
    pub operator: Pubkey,
    pub amount: u64,
}

fn require_owner(config: &GlobalConfig, owner: &Signer<'_>) -> Result<()> {
    require_keys_eq!(config.owner, owner.key(), ExchangeError::NotOwner);
    Ok(())
}

fn require_config_ready(config: &GlobalConfig) -> Result<()> {
    require_keys_neq!(config.owner, Pubkey::default(), ExchangeError::ConfigNotInitialized);
    require_keys_neq!(config.atx_mint, Pubkey::default(), ExchangeError::ConfigNotInitialized);
    Ok(())
}

fn initialize_user_points_if_needed(
    user_points: &mut Account<'_, UserPoints>,
    owner: Pubkey,
) -> Result<()> {
    if user_points.owner == Pubkey::default() {
        user_points.owner = owner;
        user_points.points = 0;
        user_points.exchange_count = 0;
        user_points.last_exchange_time = 0;
        return Ok(());
    }

    require_keys_eq!(user_points.owner, owner, ExchangeError::UnauthorizedUser);
    Ok(())
}

#[error_code]
pub enum ExchangeError {
    #[msg("Insufficient points for this exchange.")]
    InsufficientPoints,
    #[msg("The exchange feature is currently paused.")]
    ExchangePaused,
    #[msg("Only the contract owner can perform this action.")]
    NotOwner,
    #[msg("ATX token transfer failed.")]
    TransferFailed,
    #[msg("Invalid wallet or mint address.")]
    InvalidAddress,
    #[msg("The contract configuration has not been initialized.")]
    ConfigNotInitialized,
    #[msg("Math overflow or underflow detected.")]
    MathOverflow,
    #[msg("The caller is not authorized to operate on this user account.")]
    UnauthorizedUser,
    #[msg("The provided mint does not match the configured ATX mint.")]
    InvalidMint,
    #[msg("Points amount must be greater than zero.")]
    InvalidPointsAmount,
    #[msg("Token amount must be greater than zero.")]
    InvalidTokenAmount,
}
