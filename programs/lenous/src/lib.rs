use anchor_lang::prelude::*;
use anchor_spl::token::{self, InitializeAccount, Mint, Token, TokenAccount, Transfer, ID};

#[program]
pub mod perpetual_dex {
    use super::*;

    pub fn initialize_token_account(ctx: Context<InitializeTokenAccount>) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            InitializeAccount {
                account: ctx.accounts.token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );
        token::initialize_account(cpi_ctx)?;

        Ok(())
    }

    pub fn deposit_tokens(ctx: Context<DepositTokens>, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.dex_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.dex_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.dex.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn place_order(
        ctx: Context<PlaceOrder>,
        asset: Pubkey,
        position: PositionType,
        price: u64,
        margin: u64,
    ) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;

        let available_margin = user_account.usdt_balance + user_account.usdc_balance;
        require!(available_margin >= margin, ErrorCode::InsufficientMargin);

        if user_account.usdt_balance >= margin {
            user_account.usdt_balance -= margin;
        } else {
            user_account.usdc_balance -= margin - user_account.usdt_balance;
            user_account.usdt_balance = 0;
        }

        let order = Order {
            asset,
            position,
            price,
            margin_locked: margin,
            settled: false,
        };
        user_account.open_positions.push(order);

        Ok(())
    }

    pub fn settle_order(
        ctx: Context<SettleOrder>,
        asset_price: u64,
        order_index: usize,
    ) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;

        let order = &mut user_account.open_positions[order_index];
        require!(!order.settled, ErrorCode::OrderAlreadySettled);

        let result = match order.position {
            PositionType::Long => asset_price > order.price,
            PositionType::Short => asset_price < order.price,
        };

        let amount = order.margin_locked;

        if result {
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.dex_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.dex.to_account_info(),
                },
            );
            token::transfer(cpi_ctx, amount)?;
        } else {
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.dex_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            );
            token::transfer(cpi_ctx, amount)?;
        }
        order.settled = true;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct DepositTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = dex_token_account.owner == dex.key()
    )]
    pub dex_token_account: Account<'info, TokenAccount>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub dex: Signer<'info>,

    #[account(address = USDT_MINT)]
    pub usdt_mint: Account<'info, Mint>,

    #[account(address = USDC_MINT)]
    pub usdc_mint: Account<'info, Mint>,
}

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    #[account(mut)]
    pub dex: Signer<'info>,

    #[account(
        mut,
        constraint = dex_token_account.owner == dex.key()
    )]
    pub dex_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(address = USDT_MINT)]
    pub usdt_mint: Account<'info, Mint>,

    #[account(address = USDC_MINT)]
    pub usdc_mint: Account<'info, Mint>,
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = dex_token_account.owner == dex.key()
    )]
    pub dex_token_account: Account<'info, TokenAccount>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub dex: Signer<'info>,

    #[account(address = USDT_MINT)]
    pub usdt_mint: Account<'info, Mint>,

    #[account(address = USDC_MINT)]
    pub usdc_mint: Account<'info, Mint>,
}

#[derive(Accounts)]
pub struct SettleOrder<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        constraint = dex_account.owner == dex.key()
    )]
    pub dex_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub dex: Signer<'info>,
}

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub usdt_balance: u64,
    pub usdc_balance: u64,
    pub open_positions: Vec<Order>,
}

#[account]
pub struct Order {
    pub asset: Pubkey,
    pub position: PositionType,
    pub price: u64,
    pub margin_locked: u64,
    pub settled: bool,
}

#[derive(AnchorDeserialize, AnchorSerialize, PartialEq, Eq, Clone)]
pub enum PositionType {
    Long,
    Short,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient margin to place the order.")]
    InsufficientMargin,

    #[msg("Order has already been settled.")]
    OrderAlreadySettled,

    #[msg("The owner of the token account does not match the expected account.")]
    IncorrectAccountOwner,

    #[msg("Account validation failed.")]
    AccountValidationFailed,

    #[msg("An unexpected error occurred.")]
    UnexpectedError,
}

#[derive(Accounts)]
pub struct InitializeTokenAccount<'info> {
    #[account(init, payer = owner, space = 8 + TokenAccount::LEN)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub const USDT_MINT: Pubkey = Pubkey::new_from_array([
    206, 1, 14, 96, 175, 237, 178, 39, 23, 189, 99, 25, 47, 84, 20, 90, 63, 150, 90, 51, 187, 130,
    210, 199, 2, 158, 178, 206, 30, 32, 130, 100,
]);
pub const USDC_MINT: Pubkey = Pubkey::new_from_array([
    198, 250, 122, 243, 190, 219, 173, 58, 61, 101, 243, 106, 171, 201, 116, 49, 177, 187, 228,
    194, 210, 246, 224, 228, 124, 166, 2, 3, 69, 47, 93, 97,
]);
