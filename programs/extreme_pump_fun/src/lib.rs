use anchor_lang::prelude::*;

// DEVELOPMENT PLACEHOLDER ONLY. Replace with the program keypair's public key
// before building/deploying. The System Program ID is not a deployable app ID.
declare_id!("11111111111111111111111111111111");

pub const PLATFORM_FEE_BPS: u16 = 100;
pub const BPS_DENOMINATOR: u16 = 10_000;
pub const CONFIG_SEED: &[u8] = b"config";
pub const CURVE_SEED: &[u8] = b"curve";

#[program]
pub mod extreme_pump_fun {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.platform_fee_bps = PLATFORM_FEE_BPS;
        config.bump = ctx.bumps.config;
        emit!(PlatformInitialized {
            authority: config.authority,
            platform_fee_bps: config.platform_fee_bps,
        });
        Ok(())
    }

    /// Creates curve metadata only. This instruction does not mint tokens,
    /// accept SOL, transfer assets, or enable trading.
    pub fn create_curve(
        ctx: Context<CreateCurve>,
        mint: Pubkey,
        virtual_sol_reserves: u64,
        virtual_token_reserves: u64,
        creator_fee_bps: u16,
    ) -> Result<()> {
        require!(virtual_sol_reserves > 0, LaunchpadError::InvalidReserves);
        require!(virtual_token_reserves > 0, LaunchpadError::InvalidReserves);
        require!(
            creator_fee_bps < BPS_DENOMINATOR - ctx.accounts.config.platform_fee_bps,
            LaunchpadError::CombinedFeeTooHigh
        );

        let curve = &mut ctx.accounts.curve;
        curve.creator = ctx.accounts.creator.key();
        curve.mint = mint;
        curve.virtual_sol_reserves = virtual_sol_reserves;
        curve.virtual_token_reserves = virtual_token_reserves;
        curve.real_sol_reserves = 0;
        curve.real_token_reserves = 0;
        curve.creator_fee_bps = creator_fee_bps;
        curve.is_complete = false;
        curve.bump = ctx.bumps.curve;

        emit!(CurveCreated {
            creator: curve.creator,
            mint,
            creator_fee_bps,
        });
        Ok(())
    }

    /// Updates the creator fee for an existing curve. Only the curve creator
    /// may call this. Fee changes are limited so platform + creator fees < 100%.
    pub fn update_creator_fee(
        ctx: Context<UpdateCreatorFee>,
        creator_fee_bps: u16,
    ) -> Result<()> {
        require!(
            creator_fee_bps < BPS_DENOMINATOR - ctx.accounts.config.platform_fee_bps,
            LaunchpadError::CombinedFeeTooHigh
        );
        ctx.accounts.curve.creator_fee_bps = creator_fee_bps;
        emit!(CreatorFeeUpdated {
            mint: ctx.accounts.curve.mint,
            creator: ctx.accounts.curve.creator,
            creator_fee_bps,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + PlatformConfig::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, PlatformConfig>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(mint: Pubkey)]
pub struct CreateCurve<'info> {
    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, PlatformConfig>,
    #[account(
        init,
        payer = creator,
        space = 8 + BondingCurve::INIT_SPACE,
        seeds = [CURVE_SEED, mint.as_ref()],
        bump
    )]
    pub curve: Account<'info, BondingCurve>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateCreatorFee<'info> {
    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, PlatformConfig>,
    #[account(
        mut,
        seeds = [CURVE_SEED, curve.mint.as_ref()],
        bump = curve.bump,
        has_one = creator
    )]
    pub curve: Account<'info, BondingCurve>,
    pub creator: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct PlatformConfig {
    pub authority: Pubkey,
    pub platform_fee_bps: u16,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct BondingCurve {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub creator_fee_bps: u16,
    pub is_complete: bool,
    pub bump: u8,
}

#[event]
pub struct PlatformInitialized {
    pub authority: Pubkey,
    pub platform_fee_bps: u16,
}

#[event]
pub struct CurveCreated {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub creator_fee_bps: u16,
}

#[event]
pub struct CreatorFeeUpdated {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub creator_fee_bps: u16,
}

#[error_code]
pub enum LaunchpadError {
    #[msg("Virtual reserves must both be greater than zero.")]
    InvalidReserves,
    #[msg("Combined platform and creator fees must be below 10,000 basis points.")]
    CombinedFeeTooHigh,
}
