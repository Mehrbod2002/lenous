use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};

declare_id!("25TiYvDPHV63mG78NU9UAUmTcRaqB3DJ4rgiGZ34sHYC");

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    /// CHECK: This account is a token account for USDT, and the program should ensure it's the correct account during initialization.
    pub user_token_account: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This account is a token account for USDT, and the program should ensure it's the correct account during initialization.
    pub dex_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(init, payer = user, space = 8 + 32 + 32 + 32 + 8 + 8 + 1)]
    /// CHECK: manual
    pub order: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub dex_account: Account<'info, DexAccount>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub dex_account: Account<'info, DexAccount>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    /// CHECK: manual
    pub position: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    /// CHECK: This account is a token account for USDT, and the program should ensure it's the correct account during initialization.
    pub user_token_account: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This account is a token account for USDT, and the program should ensure it's the correct account during initialization.
    pub dex_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Deposit<'info> {
    pub fn into_transfer_to_dex_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, token::Transfer<'info>> {
        let cpi_accounts = token::Transfer {
            from: self.user_token_account.to_account_info(),
            to: self.dex_token_account.to_account_info(),
            authority: self.user.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

impl<'info> Withdraw<'info> {
    pub fn into_transfer_to_user_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, token::Transfer<'info>> {
        let cpi_accounts = token::Transfer {
            from: self.dex_token_account.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: self.user_account.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct GetDexTokenAccounts<'info> {
    #[account()]
    pub dex_account: Account<'info, DexAccount>,
}

#[account]
pub struct DexAccount {
    pub authority: Pubkey,
    pub usdt_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub usdt_token_account: Pubkey,
    pub usdc_token_account: Pubkey,
    pub fee_rate: u64,
    pub order_count: u64,
}

#[account]
pub struct UserAccount {
    pub user: Pubkey,
    pub margin_balance_usdt: u64,
    pub margin_balance_usdc: u64,
    // pub open_positions: Vec<Position>, // Uncomment this if it's needed
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct Position {
    pub id: u64,
    pub user: Pubkey,
    pub asset: String,
    pub entry_price: u64,
    pub leverage: u8,
    pub size: u64,
    pub is_long: bool,
    pub status: PositionStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PositionStatus {
    Open,
    Closed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum StablecoinType {
    USDT,
    USDC,
}

#[error_code]
pub enum DexError {
    #[msg("Insufficient funds for the transaction")]
    InsufficientFunds,
    #[msg("Order not found")]
    OrderNotFound,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Invalid leverage")]
    InvalidLeverage,
}

#[program]
pub mod lenous {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        usdt_mint: Pubkey,
        usdc_mint: Pubkey,
        fee_rate: u64,
    ) -> Result<()> {
        let dex_account = &mut ctx.accounts.dex_account;
        dex_account.authority = ctx.accounts.admin.key();
        dex_account.usdt_mint = usdt_mint;
        dex_account.usdc_mint = usdc_mint;
        dex_account.fee_rate = fee_rate;
        dex_account.order_count = 0;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let cpi_ctx = ctx.accounts.into_transfer_to_dex_context();
        token::transfer(cpi_ctx, amount)?;

        // let user_account = &mut ctx.accounts.user_account;
        // match stablecoin {
        //     StablecoinType::USDT => user_account.margin_balance_usdt += amount,
        //     StablecoinType::USDC => user_account.margin_balance_usdc += amount,
        // }

        Ok(())
    }

    pub fn place_order(
        ctx: Context<PlaceOrder>,
        asset: String,
        leverage: u8,
        size: u64,
        is_long: bool,
        entry_price: u64,
    ) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let dex_account = &mut ctx.accounts.dex_account;

        let margin_required = size
            .checked_div(leverage as u64)
            .ok_or(DexError::InvalidLeverage)?;

        require!(
            user_account.margin_balance_usdt >= margin_required,
            DexError::InsufficientFunds
        );

        let position = Position {
            id: dex_account.order_count,
            user: ctx.accounts.user.key(),
            asset,
            entry_price,
            leverage,
            size,
            is_long,
            status: PositionStatus::Open,
        };

        user_account.margin_balance_usdt = user_account
            .margin_balance_usdt
            .checked_sub(margin_required)
            .ok_or(DexError::InsufficientFunds)?;

        // user_account.open_positions.push(position);

        dex_account.order_count += 1;

        Ok(())
    }

    pub fn get_dex_token_accounts(ctx: Context<GetDexTokenAccounts>) -> Result<(Pubkey, Pubkey)> {
        let dex_account = &ctx.accounts.dex_account;
        Ok((
            dex_account.usdt_token_account,
            dex_account.usdc_token_account,
        ))
    }

    pub fn close_position(ctx: Context<ClosePosition>, position_id: u64, price: u64) -> Result<()> {
        //     let user_account = &mut ctx.accounts.user_account;

        //     let position = user_account
        //         .open_positions
        //         .iter_mut()
        //         .find(|p| p.id == position_id)
        //         .ok_or(DexError::OrderNotFound)?;

        //     let pnl = if position.is_long {
        //         (price as i64 - position.entry_price as i64) * position.size as i64
        //             / position.entry_price as i64
        //     } else {
        //         (position.entry_price as i64 - price as i64) * position.size as i64
        //             / position.entry_price as i64
        //     };

        //     position.status = PositionStatus::Closed;

        //     if pnl > 0 {
        //         user_account.margin_balance_usdt += pnl as u64;
        //     } else {
        //         user_account.margin_balance_usdt = user_account
        //             .margin_balance_usdt
        //             .checked_sub((-pnl) as u64)
        //             .ok_or(DexError::InsufficientFunds)?;
        //     }

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        // let user_account = &mut ctx.accounts.user_account;

        // match stablecoin {
        //     StablecoinType::USDT => {
        //         require!(
        //             user_account.margin_balance_usdt >= amount,
        //             DexError::InsufficientFunds
        //         );
        //         user_account.margin_balance_usdt -= amount;
        //     }
        //     StablecoinType::USDC => {
        //         require!(
        //             user_account.margin_balance_usdc >= amount,
        //             DexError::InsufficientFunds
        //         );
        //         user_account.margin_balance_usdc -= amount;
        //     }
        // }

        // Transfer tokens from the DEX account to the user's account
        let cpi_ctx = ctx.accounts.into_transfer_to_user_context();
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction()]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 32 + 32 + 8 + 8 + 32 + 32)]
    pub dex_account: Account<'info, DexAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    /// CHECK: This account is a token account for USDT, and the program should ensure it's the correct account during initialization.
    pub usdt_token_account: AccountInfo<'info>,
    /// CHECK: This account is a token account for USDT, and the program should ensure it's the correct account during initialization.
    pub usdc_mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
