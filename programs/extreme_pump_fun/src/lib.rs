use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer as system_transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, TransferChecked},
};

declare_id!("11111111111111111111111111111111");

pub const PLATFORM_FEE_BPS: u16 = 100;
pub const BPS: u128 = 10_000;
pub const CONFIG_SEED: &[u8] = b"config";
pub const CURVE_SEED: &[u8] = b"curve";
pub const SOL_VAULT_SEED: &[u8] = b"sol-vault";

#[program]
pub mod extreme_pump_fun {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, platform_treasury: Pubkey) -> Result<()> {
        require!(platform_treasury != Pubkey::default(), LaunchpadError::InvalidTreasury);
        let cfg = &mut ctx.accounts.config;
        cfg.authority = ctx.accounts.authority.key();
        cfg.platform_treasury = platform_treasury;
        cfg.platform_fee_bps = PLATFORM_FEE_BPS;
        cfg.bump = ctx.bumps.config;
        emit!(PlatformInitialized { authority: cfg.authority, platform_treasury, platform_fee_bps: PLATFORM_FEE_BPS });
        Ok(())
    }

    /// Creates a curve and deposits pre-minted inventory into its PDA-owned token vault.
    /// The mint must already exist; this does not mint tokens or alter mint authority.
    pub fn create_curve(ctx: Context<CreateCurve>, virtual_sol_reserves: u64, virtual_token_reserves: u64, token_inventory: u64, creator_fee_bps: u16) -> Result<()> {
        require!(virtual_sol_reserves > 0 && virtual_token_reserves > 0, LaunchpadError::InvalidReserves);
        require!(token_inventory > 0, LaunchpadError::ZeroAmount);
        validate_fees(ctx.accounts.config.platform_fee_bps, creator_fee_bps)?;
        let curve_key = ctx.accounts.curve.key();
        let curve = &mut ctx.accounts.curve;
        curve.creator = ctx.accounts.creator.key();
        curve.mint = ctx.accounts.mint.key();
        curve.virtual_sol_reserves = virtual_sol_reserves;
        curve.virtual_token_reserves = virtual_token_reserves;
        curve.real_sol_reserves = 0;
        curve.real_token_reserves = token_inventory;
        curve.creator_fee_bps = creator_fee_bps;
        curve.is_complete = false;
        curve.bump = ctx.bumps.curve;
        ctx.accounts.sol_vault.curve = curve_key;
        ctx.accounts.sol_vault.bump = ctx.bumps.sol_vault;

        let cpi = CpiContext::new(ctx.accounts.token_program.to_account_info(), TransferChecked {
            from: ctx.accounts.creator_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_vault.to_account_info(),
            authority: ctx.accounts.creator.to_account_info(),
        });
        token::transfer_checked(cpi, token_inventory, ctx.accounts.mint.decimals)?;
        emit!(CurveCreated { creator: curve.creator, mint: curve.mint, token_inventory, creator_fee_bps });
        Ok(())
    }

    pub fn buy(ctx: Context<Trade>, sol_in: u64, min_tokens_out: u64, deadline: i64) -> Result<()> {
        require!(sol_in > 0, LaunchpadError::ZeroAmount);
        check_deadline(deadline)?;
        require!(!ctx.accounts.curve.is_complete, LaunchpadError::CurveComplete);
        let (platform_fee, creator_fee, net_in) = fees(sol_in, ctx.accounts.config.platform_fee_bps, ctx.accounts.curve.creator_fee_bps)?;
        require!(net_in > 0, LaunchpadError::ZeroAmount);

        let v_sol = ctx.accounts.curve.virtual_sol_reserves as u128;
        let v_token = ctx.accounts.curve.virtual_token_reserves as u128;
        let k = v_sol.checked_mul(v_token).ok_or(LaunchpadError::MathOverflow)?;
        let new_v_sol = v_sol.checked_add(net_in as u128).ok_or(LaunchpadError::MathOverflow)?;
        let new_v_token = ceil_div(k, new_v_sol)?;
        let tokens_out = u64::try_from(v_token.checked_sub(new_v_token).ok_or(LaunchpadError::MathOverflow)?)
            .map_err(|_| error!(LaunchpadError::MathOverflow))?;
        require!(tokens_out > 0 && tokens_out >= min_tokens_out, LaunchpadError::SlippageExceeded);
        require!(tokens_out <= ctx.accounts.curve.real_token_reserves, LaunchpadError::InsufficientTokenReserve);

        system_transfer(CpiContext::new(ctx.accounts.system_program.to_account_info(), Transfer {
            from: ctx.accounts.user.to_account_info(), to: ctx.accounts.sol_vault.to_account_info(),
        }), net_in)?;
        pay_from_user(&ctx.accounts.system_program, &ctx.accounts.user, &ctx.accounts.platform_treasury, platform_fee)?;
        pay_from_user(&ctx.accounts.system_program, &ctx.accounts.user, &ctx.accounts.creator_wallet, creator_fee)?;

        let bump = [ctx.accounts.curve.bump];
        let seeds: &[&[u8]] = &[CURVE_SEED, ctx.accounts.curve.mint.as_ref(), &bump];
        let signer = &[seeds];
        let token_ctx = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), TransferChecked {
            from: ctx.accounts.token_vault.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.curve.to_account_info(),
        }, signer);
        token::transfer_checked(token_ctx, tokens_out, ctx.accounts.mint.decimals)?;

        let curve = &mut ctx.accounts.curve;
        curve.virtual_sol_reserves = u64::try_from(new_v_sol).map_err(|_| error!(LaunchpadError::MathOverflow))?;
        curve.virtual_token_reserves = u64::try_from(new_v_token).map_err(|_| error!(LaunchpadError::MathOverflow))?;
        curve.real_sol_reserves = curve.real_sol_reserves.checked_add(net_in).ok_or(LaunchpadError::MathOverflow)?;
        curve.real_token_reserves = curve.real_token_reserves.checked_sub(tokens_out).ok_or(LaunchpadError::MathOverflow)?;
        emit!(TradeExecuted { mint: curve.mint, trader: ctx.accounts.user.key(), is_buy: true, token_amount: tokens_out, sol_gross: sol_in, platform_fee, creator_fee, sol_net: net_in });
        Ok(())
    }

    pub fn sell(ctx: Context<Trade>, tokens_in: u64, min_sol_out: u64, deadline: i64) -> Result<()> {
        require!(tokens_in > 0, LaunchpadError::ZeroAmount);
        check_deadline(deadline)?;
        require!(!ctx.accounts.curve.is_complete, LaunchpadError::CurveComplete);

        let v_sol = ctx.accounts.curve.virtual_sol_reserves as u128;
        let v_token = ctx.accounts.curve.virtual_token_reserves as u128;
        let denominator = v_token.checked_add(tokens_in as u128).ok_or(LaunchpadError::MathOverflow)?;
        let gross = u64::try_from(v_sol.checked_mul(tokens_in as u128).ok_or(LaunchpadError::MathOverflow)? / denominator)
            .map_err(|_| error!(LaunchpadError::MathOverflow))?;
        let (platform_fee, creator_fee, net_out) = fees(gross, ctx.accounts.config.platform_fee_bps, ctx.accounts.curve.creator_fee_bps)?;
        require!(net_out >= min_sol_out && net_out > 0, LaunchpadError::SlippageExceeded);
        require!(gross <= ctx.accounts.curve.real_sol_reserves, LaunchpadError::InsufficientSolReserve);

        let token_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), TransferChecked {
            from: ctx.accounts.user_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        });
        token::transfer_checked(token_ctx, tokens_in, ctx.accounts.mint.decimals)?;

        let rent_floor = Rent::get()?.minimum_balance(8 + SolVault::INIT_SPACE);
        let vault_balance = ctx.accounts.sol_vault.to_account_info().lamports();
        require!(vault_balance.checked_sub(gross).ok_or(LaunchpadError::InsufficientSolReserve)? >= rent_floor, LaunchpadError::InsufficientSolReserve);
        debit_vault(&ctx.accounts.sol_vault, &ctx.accounts.user, net_out)?;
        debit_vault(&ctx.accounts.sol_vault, &ctx.accounts.platform_treasury, platform_fee)?;
        debit_vault(&ctx.accounts.sol_vault, &ctx.accounts.creator_wallet, creator_fee)?;

        let curve = &mut ctx.accounts.curve;
        curve.virtual_sol_reserves = curve.virtual_sol_reserves.checked_sub(gross).ok_or(LaunchpadError::MathOverflow)?;
        curve.virtual_token_reserves = curve.virtual_token_reserves.checked_add(tokens_in).ok_or(LaunchpadError::MathOverflow)?;
        curve.real_sol_reserves = curve.real_sol_reserves.checked_sub(gross).ok_or(LaunchpadError::InsufficientSolReserve)?;
        curve.real_token_reserves = curve.real_token_reserves.checked_add(tokens_in).ok_or(LaunchpadError::MathOverflow)?;
        emit!(TradeExecuted { mint: curve.mint, trader: ctx.accounts.user.key(), is_buy: false, token_amount: tokens_in, sol_gross: gross, platform_fee, creator_fee, sol_net: net_out });
        Ok(())
    }

    pub fn update_creator_fee(ctx: Context<UpdateCreatorFee>, creator_fee_bps: u16) -> Result<()> {
        validate_fees(ctx.accounts.config.platform_fee_bps, creator_fee_bps)?;
        ctx.accounts.curve.creator_fee_bps = creator_fee_bps;
        emit!(CreatorFeeUpdated { mint: ctx.accounts.curve.mint, creator: ctx.accounts.curve.creator, creator_fee_bps });
        Ok(())
    }

    pub fn pause_curve(ctx: Context<PauseCurve>) -> Result<()> {
        ctx.accounts.curve.is_complete = true;
        emit!(CurvePaused { mint: ctx.accounts.curve.mint, authority: ctx.accounts.authority.key() });
        Ok(())
    }
}

fn check_deadline(deadline: i64) -> Result<()> {
    require!(Clock::get()?.unix_timestamp <= deadline, LaunchpadError::Expired);
    Ok(())
}
fn validate_fees(platform: u16, creator: u16) -> Result<()> {
    require!((platform as u32) + (creator as u32) < 10_000, LaunchpadError::CombinedFeeTooHigh);
    Ok(())
}
fn fees(gross: u64, platform_bps: u16, creator_bps: u16) -> Result<(u64, u64, u64)> {
    validate_fees(platform_bps, creator_bps)?;
    let p = (gross as u128).checked_mul(platform_bps as u128).ok_or(LaunchpadError::MathOverflow)? / BPS;
    let c = (gross as u128).checked_mul(creator_bps as u128).ok_or(LaunchpadError::MathOverflow)? / BPS;
    let p = u64::try_from(p).map_err(|_| error!(LaunchpadError::MathOverflow))?;
    let c = u64::try_from(c).map_err(|_| error!(LaunchpadError::MathOverflow))?;
    let net = gross.checked_sub(p).and_then(|v| v.checked_sub(c)).ok_or(LaunchpadError::MathOverflow)?;
    Ok((p, c, net))
}
fn ceil_div(n: u128, d: u128) -> Result<u128> {
    require!(d > 0, LaunchpadError::MathOverflow);
    Ok(n.checked_add(d - 1).ok_or(LaunchpadError::MathOverflow)? / d)
}
fn pay_from_user<'info>(system_program: &Program<'info, System>, user: &Signer<'info>, recipient: &SystemAccount<'info>, amount: u64) -> Result<()> {
    if amount == 0 || user.key() == recipient.key() { return Ok(()); }
    system_transfer(CpiContext::new(system_program.to_account_info(), Transfer {
        from: user.to_account_info(), to: recipient.to_account_info(),
    }), amount)
}
fn debit_vault<'info>(vault: &Account<'info, SolVault>, recipient: &SystemAccount<'info>, amount: u64) -> Result<()> {
    if amount == 0 { return Ok(()); }
    let from = vault.to_account_info();
    let to = recipient.to_account_info();
    let from_after = from.lamports().checked_sub(amount).ok_or(LaunchpadError::InsufficientSolReserve)?;
    let to_after = to.lamports().checked_add(amount).ok_or(LaunchpadError::MathOverflow)?;
    **from.try_borrow_mut_lamports()? = from_after;
    **to.try_borrow_mut_lamports()? = to_after;
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + PlatformConfig::INIT_SPACE, seeds = [CONFIG_SEED], bump)]
    pub config: Account<'info, PlatformConfig>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct CreateCurve<'info> {
    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, PlatformConfig>,
    #[account(init, payer = creator, space = 8 + BondingCurve::INIT_SPACE, seeds = [CURVE_SEED, mint.key().as_ref()], bump)]
    pub curve: Account<'info, BondingCurve>,
    #[account(init, payer = creator, space = 8 + SolVault::INIT_SPACE, seeds = [SOL_VAULT_SEED, mint.key().as_ref()], bump)]
    pub sol_vault: Account<'info, SolVault>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut, constraint = creator_token_account.owner == creator.key(), constraint = creator_token_account.mint == mint.key())]
    pub creator_token_account: Account<'info, TokenAccount>,
    #[account(init, payer = creator, associated_token::mint = mint, associated_token::authority = curve)]
    pub token_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Trade<'info> {
    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, PlatformConfig>,
    #[account(mut, seeds = [CURVE_SEED, mint.key().as_ref()], bump = curve.bump, has_one = mint)]
    pub curve: Account<'info, BondingCurve>,
    #[account(mut, seeds = [SOL_VAULT_SEED, mint.key().as_ref()], bump = sol_vault.bump, constraint = sol_vault.curve == curve.key())]
    pub sol_vault: Account<'info, SolVault>,
    pub mint: Account<'info, Mint>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = curve)]
    pub token_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut, address = curve.creator)]
    pub creator_wallet: SystemAccount<'info>,
    #[account(mut, address = config.platform_treasury)]
    pub platform_treasury: SystemAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct UpdateCreatorFee<'info> {
    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, PlatformConfig>,
    #[account(mut, seeds = [CURVE_SEED, curve.mint.as_ref()], bump = curve.bump, has_one = creator)]
    pub curve: Account<'info, BondingCurve>,
    pub creator: Signer<'info>,
}
#[derive(Accounts)]
pub struct PauseCurve<'info> {
    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = authority)]
    pub config: Account<'info, PlatformConfig>,
    pub authority: Signer<'info>,
    #[account(mut, seeds = [CURVE_SEED, curve.mint.as_ref()], bump = curve.bump)]
    pub curve: Account<'info, BondingCurve>,
}
#[account]
#[derive(InitSpace)]
pub struct PlatformConfig { pub authority: Pubkey, pub platform_treasury: Pubkey, pub platform_fee_bps: u16, pub bump: u8 }
#[account]
#[derive(InitSpace)]
pub struct BondingCurve {
    pub creator: Pubkey, pub mint: Pubkey, pub virtual_sol_reserves: u64, pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64, pub real_token_reserves: u64, pub creator_fee_bps: u16, pub is_complete: bool, pub bump: u8,
}
#[account]
#[derive(InitSpace)]
pub struct SolVault { pub curve: Pubkey, pub bump: u8 }
#[event]
pub struct PlatformInitialized { pub authority: Pubkey, pub platform_treasury: Pubkey, pub platform_fee_bps: u16 }
#[event]
pub struct CurveCreated { pub creator: Pubkey, pub mint: Pubkey, pub token_inventory: u64, pub creator_fee_bps: u16 }
#[event]
pub struct TradeExecuted { pub mint: Pubkey, pub trader: Pubkey, pub is_buy: bool, pub token_amount: u64, pub sol_gross: u64, pub platform_fee: u64, pub creator_fee: u64, pub sol_net: u64 }
#[event]
pub struct CreatorFeeUpdated { pub mint: Pubkey, pub creator: Pubkey, pub creator_fee_bps: u16 }
#[event]
pub struct CurvePaused { pub mint: Pubkey, pub authority: Pubkey }
#[error_code]
pub enum LaunchpadError {
    #[msg("Invalid platform treasury address.")] InvalidTreasury,
    #[msg("Virtual reserves must be positive.")] InvalidReserves,
    #[msg("Trade amount must be positive.")] ZeroAmount,
    #[msg("Combined fees must be below 10,000 basis points.")] CombinedFeeTooHigh,
    #[msg("Curve has been paused or completed.")] CurveComplete,
    #[msg("Trade quote is below the caller's minimum output.")] SlippageExceeded,
    #[msg("Insufficient real token reserve.")] InsufficientTokenReserve,
    #[msg("Insufficient SOL reserve.")] InsufficientSolReserve,
    #[msg("Arithmetic overflow or underflow.")] MathOverflow,
    #[msg("Transaction deadline has passed.")] Expired,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_split_is_separate_and_conserves_gross() {
        let (platform, creator, net) = fees(1_000_000, 100, 200).unwrap();
        assert_eq!(platform, 10_000);
        assert_eq!(creator, 20_000);
        assert_eq!(net, 970_000);
        assert_eq!(platform + creator + net, 1_000_000);
    }

    #[test]
    fn fees_reject_combined_rate_at_or_above_one_hundred_percent() {
        assert!(validate_fees(100, 9_900).is_err());
        assert!(validate_fees(100, 9_899).is_ok());
    }

    #[test]
    fn ceil_div_rounds_up() {
        assert_eq!(ceil_div(10, 3).unwrap(), 4);
        assert_eq!(ceil_div(12, 3).unwrap(), 4);
    }

    #[test]
    fn buy_quote_rounding_preserves_constant_product_floor() {
        let virtual_sol = 1_000_000u128;
        let virtual_tokens = 10_000_000u128;
        let net_input = 100_000u128;
        let invariant = virtual_sol * virtual_tokens;
        let new_sol = virtual_sol + net_input;
        let new_tokens = ceil_div(invariant, new_sol).unwrap();
        assert!(new_sol * new_tokens >= invariant);
        assert!(new_tokens < virtual_tokens);
    }
}
