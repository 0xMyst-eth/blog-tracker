use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction::transfer;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod list_tracker {
    use super::*;
    pub fn initialize(ctx: Context<InitializeList>, name: String, capacity: u16, list_bump: u8) -> ProgramResult {
        let list_to_init = &mut ctx.accounts.list_to_init;
        list_to_init.list_owner = *ctx.accounts.list_creator.to_account_info().key;
        list_to_init.name = name;
        list_to_init.capacity = capacity;
        list_to_init.bump = list_bump;
        Ok(())
    }

    pub fn add(ctx: Context<CreateItem>, _list_name: String, item_name: String, bounty: u64)-> ProgramResult{
        let item_creator = &ctx.accounts.item_creator;
        let list = &mut ctx.accounts.list_account;
        let item = &mut ctx.accounts.item_account;
        
        
        if list.items.len() >= list.capacity as usize {
            return Err(ToDoListError::ListFull.into())
        }


        item.name = item_name;
        item.creator = ctx.accounts.item_creator.key();
        list.items.push(item.to_account_info().key());

        let account_lamports = **item.to_account_info().lamports.borrow();
        let remaining_to_transfer = bounty.checked_sub(account_lamports)
                                        .ok_or(ToDoListError::BountyTooSmall)?;

        if remaining_to_transfer >0 {
            invoke (
                &transfer(
                    item_creator.to_account_info().key,
                    item.to_account_info().key,
                    remaining_to_transfer
                ),
                &[
                    item_creator.to_account_info(),
                    item.to_account_info(),
                    
                    ctx.accounts.system_program.to_account_info()

                ]
            )?
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, capacity: u16, list_bump: u8 )]
pub struct InitializeList<'info> {
    #[account(
        init,
        payer = list_creator,
        space = List::space(&name, capacity),
        
        seeds=[
          
            b"todolist",
            list_creator.to_account_info().key.as_ref(),
            cut_seed(&name)
            
        ],
        bump = list_bump
        
    )]
    pub list_to_init: Account<'info,List>,
    pub list_creator: Signer<'info>,
    pub system_program: Program<'info,System>,

}

#[derive(Accounts)]
#[instruction(list_name: String, item_name: String, bounty: u64)]
pub struct CreateItem<'info>{

    #[account(
        mut,
        has_one = list_owner,
        seeds = [
            b"todolist",
            list_owner.to_account_info().key.as_ref(),
            cut_seed(&list_name),
        ],
        bump = list_account.bump,
    )]
    pub list_account: Account<'info,List>,
    
    pub list_owner: AccountInfo<'info>, 
    #[account(
        init,
        payer = item_creator,
        space = Item::space(&item_name)       
    )]
    pub item_account: Account<'info,Item>,
    pub item_creator: Signer<'info>,
    
    
    pub system_program: Program<'info,System>
}



#[account]
pub struct List{
    pub bump: u8,
    pub list_owner: Pubkey,
    pub capacity: u16,
    pub items: Vec<Pubkey>,
    pub name: String
}

#[account]
pub struct Item{
    pub name: String,
    pub creator: Pubkey,
    pub creator_finish: bool,
    pub list_owner_finish: bool
}

impl List{
    fn space(name: &str, capacity: u16) -> usize{
        8 + 1 + 32 + 2
            + 4  + name.len()
            + 9000
    }
}

impl Item{
    fn space(name: &str)-> usize{
        8 + 32 + 1 + 1
            + 4 + name.len()
    }
}



fn cut_seed(seed: &str)-> &[u8]{
    let b = seed.as_bytes();
    if b.len() > 32{
        &b[0..32]
    }else{
        b
    }
}

#[error]
pub enum ToDoListError{
    #[msg("This List is full!")]
    ListFull,
    #[msg("Bounty must be enough to mark account rent-exempt")]
    BountyTooSmall,
    #[msg("Only the list owner or item creator may cancel an item")]
    CancelPermissions,
    #[msg("Only the list owner or item creator may finish an item")]
    FinishPermissions,
    #[msg("Item does not belong to this todo list")]
    ItemNotFound,
    #[msg("Specified list owner does not match the pubkey in the list")]
    WrongListOwner,
    #[msg("Specified item creator does not match the pubkey in the item")]
    WrongItemCreator,
}
