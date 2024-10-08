use anchor_lang::prelude::*;
use anchor_spl::{metadata::{mpl_token_metadata::instructions::
{ThawDelegatedAccountCpi, ThawDelegatedAccountCpiAccounts}, MasterEditionAccount,
Metadata, MetadataAccount}, token::{revoke, Revoke, Mint, Token, TokenAccount}};
use crate::state:: {UserAccount, StakeAccount,Competition};
use crate::error::{UserError,CompetitionError};
#[derive(Accounts)]
pub struct Exit<'info>{
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds=[b"user".as_ref(), user.key().as_ref()],
        bump=user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        seeds=[b"competition",competition.number.to_le_bytes().as_ref(),competition.admin.key().as_ref()],
        bump
    )]
    pub competition: Box<Account<'info, Competition>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub mint_ata: Account<'info, TokenAccount>,
    #[account(
        seeds= [
            b"metadata",
            metadata_program.key().as_ref(),
            mint.key().as_ref(),
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub metadata:Account<'info, MetadataAccount>,
    
    #[account(
        seeds= [
            b"metadata",
            metadata_program.key().as_ref(),
            mint.key().as_ref(),
            b"edition"
            ],
            seeds::program = metadata_program.key(),
            bump,
        )]
    pub edition:Account<'info, MasterEditionAccount>,

    #[account(
        seeds = [b"stake".as_ref(), user.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub metadata_program:Program<'info, Metadata>,
}

impl<'info> Exit<'info>{

    pub fn exit(&mut self) -> Result<()> {  
        require!(self.competition.can_claim == true, CompetitionError::CantClaim);
        require!(self.user_account.nft_in_competition == true, UserError::NotEntered);

        // set the user account to be out of competition
        self.user_account.paid_entry_fees=false;
        self.user_account.nft_in_competition=false;

        let delegate = &self.stake_account.to_account_info();
        let token_account = &self.mint_ata.to_account_info();
        let edition = &self.edition.to_account_info();
        let mint = &self.mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();

        ThawDelegatedAccountCpi::new(
            metadata_program,
            ThawDelegatedAccountCpiAccounts{
                delegate,
                token_account,
                edition,
                mint,
                token_program,
            },
        ).invoke()?;

        
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Revoke {
                source: self.mint_ata.to_account_info(),
                authority: self.stake_account.to_account_info(),
            };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        revoke(cpi_ctx)?;

        // reset stake account
        self.stake_account.mint = Pubkey::from([0u8; 32]);
        self.stake_account.votes = 0;

        Ok(())
    }
}