pub mod utils;
use borsh::{BorshDeserialize,BorshSerialize};
use arrayref::{array_ref};
use {
    // crate::utils::*,
    anchor_lang::{
        prelude::*,
        AnchorDeserialize,
        AnchorSerialize,
        Key,
        Discriminator,
        solana_program::{
            program::{invoke,invoke_signed},
            program_pack::Pack,
            system_instruction,
            system_program
        }      
    },
    metaplex_token_metadata::{
        instruction::{create_metadata_accounts,create_master_edition,update_metadata_accounts},
        state::{
            MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
        },
    },
    spl_token::{
        state,
        instruction::transfer,
    },
    std::cell::Ref,
};
declare_id!("Ah2FHrgj7yNxvriY8CCXq7KYnKAHUHwHCzre2LzNTxzc");

#[program]
pub mod solana_anchor {
    use super::*;

    pub fn init_config(
        ctx : Context<InitConfig>,
        _max_number_of_lines : u32,
        _config_data : ConfigData,
        ) ->ProgramResult {
        msg!("+ init_config");
        let config_info = &mut ctx.accounts.config;
        let mut new_data = Config::discriminator().try_to_vec().unwrap();
        new_data.append(&mut (*ctx.accounts.authority.key).try_to_vec().unwrap());
        new_data.append(&mut (_max_number_of_lines).try_to_vec().unwrap());
        let mut config_data = _config_data;
        let mut array_of_zeroes = vec![];
        while array_of_zeroes.len() < MAX_SYMBOL_LENGTH - config_data.symbol.len() {
            array_of_zeroes.push(0u8);
        }
        let new_symbol = config_data.symbol.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
        config_data.symbol = new_symbol;
        new_data.append(&mut config_data.try_to_vec().unwrap());
        // new_data.append(&mut config_data.symbol.try_to_vec().unwrap());
        let mut data = config_info.data.borrow_mut();
        for i in 0..new_data.len(){
            data[i] = new_data[i];
        }
        let vec_start = 8 + CONFIG_SIZE;
        let as_bytes = (0 as u32).to_le_bytes();
        for i in 0..4 {
            data[vec_start+i] = as_bytes[i];
        }
        Ok(())
    }

    pub fn update_config(
        ctx : Context<UpdateConfig>,
        _config_data : ConfigData
        ) -> ProgramResult {
        msg!("+ update_config");
        let authority = get_authority(&ctx.accounts.config)?;
        if authority != *ctx.accounts.authority.key {
            return Err(PoolError::InvalidAuthority.into());
        }
        let config_info = &mut ctx.accounts.config;
        let mut data = config_info.data.borrow_mut();
        let mut config_data = _config_data;
        let mut array_of_zeroes = vec![];
        while array_of_zeroes.len() < MAX_SYMBOL_LENGTH - config_data.symbol.len() {
            array_of_zeroes.push(0u8);
        }
        let new_symbol = config_data.symbol.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
        config_data.symbol = new_symbol;
        
        data[44..44+CONFIG_DATA_SIZE].copy_from_slice(&mut config_data.try_to_vec().unwrap());
        Ok(())
    }

    pub fn add_config_lines(
        ctx : Context<AddConfigLines>,
        config_lines : Vec<ConfigLine>
        ) ->ProgramResult {
        msg!("+ add_config_lines");
        let authority = get_authority(&ctx.accounts.config)?;
        if authority != *ctx.accounts.authority.key {
            return Err(PoolError::InvalidAuthority.into());
        }        
        let current_count = get_config_count(&ctx.accounts.config.data.borrow())?;
        let mut data = ctx.accounts.config.data.borrow_mut();
        let mut fixed_config_lines = vec![];
        for line in &config_lines {
            let mut array_of_zeroes = vec![];
            let mut count_limit = MAX_NAME_LENGTH - line.name.len();
            while array_of_zeroes.len() < count_limit{
                array_of_zeroes.push(0u8);
            }
            let name = line.name.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();

            let mut array_of_zeroes = vec![];
            count_limit = MAX_URI_LENGTH - line.uri.len();
            while array_of_zeroes.len() < count_limit {
                array_of_zeroes.push(0u8);
            }
            let uri = line.uri.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
            fixed_config_lines.push(ConfigLine {name, uri})
        }
        let as_vec = fixed_config_lines.try_to_vec()?;
        let serialized : &[u8] = &as_vec.as_slice()[4..];
        let position = 8 + CONFIG_SIZE + 4 + current_count as usize * CONFIG_LINE_SIZE;
        let array_slice : &mut[u8] = &mut data[position..position+fixed_config_lines.len()*CONFIG_LINE_SIZE];
        array_slice.copy_from_slice(serialized);

        let new_count : u32 = current_count as u32 + fixed_config_lines.len() as u32;
        data[8+CONFIG_SIZE..8+CONFIG_SIZE+4].copy_from_slice(&(new_count as u32).to_le_bytes());

        Ok(())
    }

    pub fn update_config_line(
        ctx : Context<AddConfigLines>,
        _index : u32,
        _config_line : ConfigLine
        ) -> ProgramResult {
        msg!("+ update_config_line");
        let authority = get_authority(&ctx.accounts.config)?;
        if authority != *ctx.accounts.authority.key{
            return Err(PoolError::InvalidAuthority.into());
        } 
        let current_count = get_config_count(&ctx.accounts.config.data.borrow())?;
        if _index >= current_count as u32{
            return Err(PoolError::InvalidIndex.into());
        }
        let mut data = ctx.accounts.config.data.borrow_mut();
        let mut config_line = _config_line;
        let mut array_of_zeroes = vec![];
        while array_of_zeroes.len() < MAX_NAME_LENGTH - config_line.name.len(){
            array_of_zeroes.push(0u8);
        }
        let name = config_line.name.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
        config_line.name = name;
        let mut array_of_zeroes = vec![];
        while array_of_zeroes.len() < MAX_URI_LENGTH - config_line.uri.len(){
            array_of_zeroes.push(0u8);
        }
        let uri = config_line.uri.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
        config_line.uri = uri;
        let position = 8 + CONFIG_SIZE + 4 + _index as usize * CONFIG_LINE_SIZE;
        data[position..position+CONFIG_LINE_SIZE].copy_from_slice(&mut config_line.try_to_vec().unwrap());
        Ok(())
    }

    pub fn init_pool(
        ctx : Context<InitPool>,
        _bump : u8,
        _update_authority : Pubkey,
        _scoby_wallet : Pubkey,
        _legendary : Pubkey,
        _redlist_black : Pubkey,
        _redlist_steel : Pubkey,
        _redlist_gold : Pubkey,
        _minting_price : u64,
        _royalty_for_minting : Royalty,
        _royalty_for_trading : Royalty,
        ) ->ProgramResult {
        msg!("+ init_pool");
        let pool = &mut ctx.accounts.pool;
        pool.owner = *ctx.accounts.owner.key;
        pool.rand = *ctx.accounts.rand.key;
        pool.config = *ctx.accounts.config.key;
        pool.count_minting = 0;
        pool.count_group_1 = 0;
        pool.count_group_2 = 0;
        pool.count_group_3 = 0;
        pool.count_group_4 = 0;
        pool.count_group_5 = 0;
        pool.count_group_6 = 0;

        pool.minting_price = _minting_price;
        pool.update_authority = _update_authority;
        pool.scoby_wallet = _scoby_wallet;
        pool.legendary = _legendary;
        pool.redlist_black = _redlist_black;
        pool.redlist_steel = _redlist_steel;
        pool.redlist_gold = _redlist_gold;
        pool.royalty_for_minting = _royalty_for_minting;
        pool.royalty_for_trading = _royalty_for_trading;
        pool.bump = _bump;
        Ok(())
    }

    pub fn update_pool(
        ctx : Context<UpdatePool>,
        _update_authority : Pubkey,
        _scoby_wallet : Pubkey,
        _legendary : Pubkey,
        _redlist_black : Pubkey,
        _redlist_steel : Pubkey,
        _redlist_gold : Pubkey,
        _minting_price : u64,
        _royalty_for_minting : Royalty,
        _royalty_for_trading : Royalty,
        ) -> ProgramResult {
        msg!("+ update_pool");
        let pool = &mut ctx.accounts.pool;
        if pool.owner != *ctx.accounts.owner.key{
            msg!("Invalid pool owner");
            return Err(PoolError::InvalidOwner.into());
        }
        pool.update_authority = _update_authority;
        pool.minting_price = _minting_price;
        pool.scoby_wallet = _scoby_wallet;
        pool.legendary = _legendary;
        pool.redlist_black = _redlist_black;
        pool.redlist_steel = _redlist_steel;
        pool.redlist_gold = _redlist_gold;
        pool.royalty_for_minting = _royalty_for_minting;
        pool.royalty_for_trading = _royalty_for_trading;
        Ok(())
    }


    pub fn mint(
        ctx : Context<Mint>,
        _bump : u8,
        _fake_id_hold : bool,
        ) -> ProgramResult {
        msg!("+ mint");
        let pool = &mut ctx.accounts.pool;
        let config_data = get_config_data(&ctx.accounts.config)?;
        // if pool.count_minting == 0 {
        //     return Err(PoolError::InvalidCreatingRoot.into());
        // }
        // let config_line = get_config_line(&ctx.accounts.config, 1)?;

        // init nft
        let nft_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.nft_account.data.borrow())?;

        let nft_mint = nft_account.mint;

        let metadata_extended = &mut ctx.accounts.metadata_extended;

        
        // init creator
        let creator_nft_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.creator_nft_account.data.borrow())?;
        
        // init parent
        let parent_nft_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.parent_nft_account.data.borrow())?;

        msg!("+ prepare done");
        
        if nft_account.owner != *ctx.accounts.owner.key 
            || nft_account.amount != 1 
            || nft_account.mint != nft_mint {
            return Err(PoolError::InvalidMintPrerequirement.into());
        }
       
        msg!("+ nft done");

        if parent_nft_account.owner != *ctx.accounts.parent_nft_owner.key
            || parent_nft_account.amount != 1 {
            return Err(PoolError::InvalidOldestMintRequirement.into());
        }

        let parent_nft_owner = parent_nft_account.owner;
        msg!("+ parent done");

        // if grand_parent_nft_account.amount != 1 {
        //     return Err(PoolError::InvalidOldestMintRequirement.into());
        // }

        let grand_parent_nft_owner = *ctx.accounts.grand_parent_nft_owner.key;
        msg!("+ grand parent done");

        // if grand_grand_parent_nft_account.amount != 1 {
        //     return Err(PoolError::InvalidOldestMintRequirement.into());
        // }

        let grand_grand_parent_nft_owner = *ctx.accounts.grand_grand_parent_nft_owner.key;
        msg!("+ grand_grand parent done");

        // if grand_grand_grand_parent_nft_account.amount != 1 {
        //     return Err(PoolError::InvalidOldestMintRequirement.into());
        // }

        let grand_grand_grand_parent_nft_owner = *ctx.accounts.grand_grand_grand_parent_nft_owner.key;
        msg!("+ grand_grand_grand parent done");

        if pool.scoby_wallet != *ctx.accounts.scoby_wallet.key {
            return Err(PoolError::InvalidPoolWallet.into());
        }

        let scoby_wallet = pool.scoby_wallet;
        msg!("valid scoby");

        if creator_nft_account.amount != 1 
            || creator_nft_account.owner != *ctx.accounts.creator_wallet.key {
            return Err(PoolError::InvalidCreatorWallet.into());
        }
        let creator_wallet = creator_nft_account.owner;
        msg!("+ creator done");

        // if creator_scout_nft_account.mint != pool.creator_scout
        //     || creator_scout_nft_account.amount != 1 {
        //     // || creator_scout_nft_account.owner != *ctx.accounts.creator_scout_wallet.key {
        //     return Err(PoolError::InvalidCreatorWallet.into());
        // }
        // let creator_scout_wallet = creator_scout_nft_account.owner;
        // msg!("+ creator scout done");

        // if ctx.accounts.owner.lamports() < pool.minting_price {
        //     return Err(PoolError::NotEnoughSol.into());
        // }

        msg!("enough sol");
        
        let mut discount : u64 = 100;
        
        let mut config_line = get_config_line(&ctx.accounts.config, 5)?;  // group 6

        let mut group_minting_count : u32 = pool.count_group_6;

        let mut group_name : String = "Open".to_string();

        
        if _fake_id_hold
        {
            discount = 90;

            config_line = get_config_line(&ctx.accounts.config, 4)?; // group 5

            group_minting_count = pool.count_group_5;

            group_name = "FID".to_string();
        }

        msg!("discount");
        msg!(&discount.to_string());

        if *ctx.accounts.owner.key != scoby_wallet {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.scoby_wallet.key,
                    pool.minting_price * pool.royalty_for_minting.scoby as u64 / 10000 * discount / 100
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.scoby_wallet.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }

        

        if *ctx.accounts.owner.key != creator_wallet {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.creator_wallet.key,
                    pool.minting_price * pool.royalty_for_minting.creator as u64 / 10000 * discount / 100
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.creator_wallet.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }

        if *ctx.accounts.owner.key != parent_nft_owner {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.parent_nft_owner.key,
                    pool.minting_price * pool.royalty_for_minting.parent as u64 / 10000 * discount / 100 
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.parent_nft_owner.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }

        if *ctx.accounts.owner.key != grand_parent_nft_owner {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.grand_parent_nft_owner.key,
                    pool.minting_price * pool.royalty_for_minting.grand_parent as u64 / 10000 * discount / 100 
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.grand_parent_nft_owner.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }

        if *ctx.accounts.owner.key != grand_grand_parent_nft_owner {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.grand_grand_parent_nft_owner.key,
                    pool.minting_price * pool.royalty_for_minting.grand_grand_parent as u64 / 10000 * discount / 100 
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.grand_grand_parent_nft_owner.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }

        msg!("grand grand parent");
        if *ctx.accounts.owner.key != grand_grand_grand_parent_nft_owner {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.grand_grand_grand_parent_nft_owner.key,
                    pool.minting_price * pool.royalty_for_minting.grand_grand_grand_parent as u64 / 10000 * discount / 100 
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.grand_grand_grand_parent_nft_owner.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }
        
        msg!("sol transfer done");

        // let mut royalty_scoby = pool.royalty_for_trading.scoby as u8;
        // let mut royalty_creator_scout = pool.royalty_for_trading.creator_scout as u8;
        let mut royalty_creator = pool.royalty_for_trading.creator as u8;
        let mut royalty_parent = pool.royalty_for_trading.parent as u8;
        let mut royalty_grand_parent = pool.royalty_for_trading.grand_parent as u8;
        let mut royalty_grand_grand_parent = pool.royalty_for_trading.grand_grand_parent as u8;
        let mut royalty_grand_grand_grand_parent = pool.royalty_for_trading.grand_grand_grand_parent as u8;
        
        if creator_wallet ==  parent_nft_owner {
            royalty_creator += royalty_parent;
            royalty_parent = 0;
        }
        
        if creator_wallet ==  grand_parent_nft_owner {
            royalty_creator += royalty_grand_parent;
            royalty_grand_parent = 0;
        }

        if creator_wallet ==  grand_grand_parent_nft_owner {
            royalty_creator += royalty_grand_grand_parent;
            royalty_grand_grand_parent = 0;
        }

        if creator_wallet ==  grand_grand_grand_parent_nft_owner {
            royalty_creator += royalty_grand_grand_grand_parent;
            royalty_grand_grand_grand_parent = 0;
        }

        if parent_nft_owner ==  grand_parent_nft_owner {
            royalty_parent += royalty_grand_parent;
            royalty_grand_parent = 0;
        }

        if parent_nft_owner ==  grand_grand_parent_nft_owner {
            royalty_parent += royalty_grand_grand_parent;
            royalty_grand_grand_parent = 0;
        }

        if parent_nft_owner ==  grand_grand_grand_parent_nft_owner {
            royalty_parent += royalty_grand_grand_grand_parent;
            royalty_grand_grand_grand_parent = 0;
        }

        if grand_parent_nft_owner ==  grand_grand_parent_nft_owner {
            royalty_grand_parent += royalty_grand_grand_parent;
            royalty_grand_grand_parent = 0;
        }

        if grand_parent_nft_owner ==  grand_grand_grand_parent_nft_owner {
            royalty_grand_parent += royalty_grand_grand_grand_parent;
            royalty_grand_grand_grand_parent = 0;
        }

        if grand_grand_parent_nft_owner ==  grand_grand_grand_parent_nft_owner {
            royalty_grand_grand_parent += royalty_grand_grand_grand_parent;
            royalty_grand_grand_grand_parent = 0;
        }

        let mut creators : Vec<metaplex_token_metadata::state::Creator> = 
            vec![metaplex_token_metadata::state::Creator{
                address : pool.key(),
                verified : true,
                share : 0,
            }];

        if royalty_creator != 0{
            creators.push(metaplex_token_metadata::state::Creator{
                address : creator_wallet,
                verified : false,
                share : royalty_creator as u8,
            });
        }

        if royalty_parent != 0{
            creators.push(metaplex_token_metadata::state::Creator{
                address : parent_nft_owner,
                verified : false,
                share : royalty_parent as u8,
            });
        }

        if royalty_grand_parent != 0{
            creators.push(metaplex_token_metadata::state::Creator{
                address : grand_parent_nft_owner,
                verified : false,
                share : royalty_grand_parent as u8,
            });
        }

        if royalty_grand_grand_parent != 0{
            creators.push(metaplex_token_metadata::state::Creator{
                address : grand_grand_parent_nft_owner,
                verified : false,
                share : royalty_grand_grand_parent as u8,
            });
        }

        if royalty_grand_grand_grand_parent != 0{
            creators.push(metaplex_token_metadata::state::Creator{
                address : grand_grand_grand_parent_nft_owner,
                verified : false,
                share : royalty_grand_grand_grand_parent as u8,
            });
        }

        // let sparkle_heart = vec![0u8];
        let substring: String ="Hellbenders ".to_owned() + &group_name.to_owned() + &" Spawn #".to_owned() + &group_minting_count.to_string();
        
        // msg!(&config_line.name.clone().replace(std::str::from_utf8(&vec![0u8]).unwrap(), "").len().to_string());
        // msg!(&substring);
        // msg!(&substring.len().to_string());

        let pool_seeds = &[pool.rand.as_ref(),&[pool.bump]];
        invoke_signed(
            &create_metadata_accounts(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.nft_mint.key,
                *ctx.accounts.owner.key,
                *ctx.accounts.owner.key,
                pool.key(),
                substring,
                config_data.symbol.clone(),
                config_line.uri,
                Some(creators),
                config_data.seller_fee,
                true,
                true,
            ),
            &[
                ctx.accounts.metadata.clone(),
                ctx.accounts.nft_mint.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.token_metadata_program.clone(),
                ctx.accounts.token_program.clone(),
                ctx.accounts.system_program.clone(),
                ctx.accounts.rent.to_account_info().clone(),
                pool.to_account_info().clone(),
            ],
            &[pool_seeds],
        )?;
        invoke_signed(
            &create_master_edition(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.master_edition.key,
                *ctx.accounts.nft_mint.key,
                pool.key(),
                *ctx.accounts.owner.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.owner.key,
                None
            ),
            &[
                ctx.accounts.master_edition.clone(),
                ctx.accounts.nft_mint.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.metadata.clone(),
                ctx.accounts.token_metadata_program.clone(),
                ctx.accounts.token_program.clone(),
                ctx.accounts.system_program.clone(),
                ctx.accounts.rent.to_account_info().clone(),
                pool.to_account_info().clone(),
            ],
            &[pool_seeds]
        )?;
        invoke_signed(
            &update_metadata_accounts(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata.key,
                pool.key(),
                Some(pool.update_authority),
                None,
                Some(true),
            ),
            &[
                ctx.accounts.token_metadata_program.clone(),
                ctx.accounts.metadata.clone(),
                pool.to_account_info().clone(),
            ],
            &[pool_seeds]
        )?;
        metadata_extended.mint = nft_mint;
        metadata_extended.minter = *ctx.accounts.owner.key;
        metadata_extended.parent_nfp = parent_nft_account.mint;
        metadata_extended.number = pool.count_minting;
        metadata_extended.bump = _bump;
        pool.count_minting = pool.count_minting + 1;

        if group_name == "FID"
        {
            pool.count_group_5 = pool.count_group_5 + 1;
        } else {
            pool.count_group_6 = pool.count_group_6 + 1;
        } 

        Ok(())
    }

    pub fn mint_with_redlist(
        ctx : Context<MintWithRedlist>,
        _bump : u8,
        _fake_id_hold : bool,
        ) -> ProgramResult {
        msg!("+ mint");
        let pool = &mut ctx.accounts.pool;
        let config_data = get_config_data(&ctx.accounts.config)?;
        // if pool.count_minting == 0 {
        //     return Err(PoolError::InvalidCreatingRoot.into());
        // }
        // let config_line = get_config_line(&ctx.accounts.config, 1)?;

        // init nft
        let nft_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.nft_account.data.borrow())?;

        let nft_mint = nft_account.mint;

        let metadata_extended = &mut ctx.accounts.metadata_extended;

        
        // init creator
        let creator_nft_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.creator_nft_account.data.borrow())?;
        
        // init parent
        let parent_nft_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.parent_nft_account.data.borrow())?;

        msg!("+ prepare done");
        
        if nft_account.owner != *ctx.accounts.owner.key 
            || nft_account.amount != 1 
            || nft_account.mint != nft_mint {
            return Err(PoolError::InvalidMintPrerequirement.into());
        }
       
        msg!("+ nft done");

        if parent_nft_account.owner != *ctx.accounts.parent_nft_owner.key
            || parent_nft_account.amount != 1 {
            return Err(PoolError::InvalidOldestMintRequirement.into());
        }

        let parent_nft_owner = parent_nft_account.owner;
        msg!("+ parent done");

        // if grand_parent_nft_account.amount != 1 {
        //     return Err(PoolError::InvalidOldestMintRequirement.into());
        // }

        let grand_parent_nft_owner = *ctx.accounts.grand_parent_nft_owner.key;
        msg!("+ grand parent done");

        // if grand_grand_parent_nft_account.amount != 1 {
        //     return Err(PoolError::InvalidOldestMintRequirement.into());
        // }

        let grand_grand_parent_nft_owner = *ctx.accounts.grand_grand_parent_nft_owner.key;
        msg!("+ grand_grand parent done");

        // if grand_grand_grand_parent_nft_account.amount != 1 {
        //     return Err(PoolError::InvalidOldestMintRequirement.into());
        // }

        let grand_grand_grand_parent_nft_owner = *ctx.accounts.grand_grand_grand_parent_nft_owner.key;
        msg!("+ grand_grand_grand parent done");

        if pool.scoby_wallet != *ctx.accounts.scoby_wallet.key {
            return Err(PoolError::InvalidPoolWallet.into());
        }

        let scoby_wallet = pool.scoby_wallet;
        msg!("valid scoby");

        if creator_nft_account.amount != 1 
            || creator_nft_account.owner != *ctx.accounts.creator_wallet.key {
            return Err(PoolError::InvalidCreatorWallet.into());
        }
        let creator_wallet = creator_nft_account.owner;
        msg!("+ creator done");

        // if creator_scout_nft_account.mint != pool.creator_scout
        //     || creator_scout_nft_account.amount != 1 {
        //     // || creator_scout_nft_account.owner != *ctx.accounts.creator_scout_wallet.key {
        //     return Err(PoolError::InvalidCreatorWallet.into());
        // }
        // let creator_scout_wallet = creator_scout_nft_account.owner;
        // msg!("+ creator scout done");

        // if ctx.accounts.owner.lamports() < pool.minting_price {
        //     return Err(PoolError::NotEnoughSol.into());
        // }

        msg!("enough sol");
        
        let mut discount : u64 = 0;
        let redlist_token_account : state::Account = state::Account::unpack_from_slice(&ctx.accounts.redlist_token_account.data.borrow())?;

        let mut config_line = get_config_line(&ctx.accounts.config, 0)?; // group 1

        let mut group_minting_count : u32 = pool.count_group_1;

        let mut group_name : String = "Legendary".to_string();

        if redlist_token_account.mint == pool.redlist_gold {
            
            config_line = get_config_line(&ctx.accounts.config, 1)?; // group 2

            group_minting_count = pool.count_group_2;

            group_name = "Gold".to_string();
            
            discount = 20;
        } else if redlist_token_account.mint == pool.redlist_steel {
            
            config_line = get_config_line(&ctx.accounts.config, 2)?; // group 3
            
            group_minting_count = pool.count_group_3;

            group_name = "Steel".to_string();

            discount = 15;
        } else if redlist_token_account.mint == pool.redlist_black {
            
            config_line = get_config_line(&ctx.accounts.config, 3)?; // group 4 

            group_minting_count = pool.count_group_4;

            group_name = "Black".to_string();

            discount = 10;
        }

        
        if _fake_id_hold {
            discount = 100 - (discount * 2);
        } else {
            discount = 100 - discount;
        }

        if redlist_token_account.mint == pool.legendary {
            
            discount = 50;
        }  

        msg!("discount");
        msg!(&discount.to_string());

        if *ctx.accounts.owner.key != scoby_wallet {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.scoby_wallet.key,
                    pool.minting_price * pool.royalty_for_minting.scoby as u64 / 10000 * discount / 100
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.scoby_wallet.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }

        

        if *ctx.accounts.owner.key != creator_wallet {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.creator_wallet.key,
                    pool.minting_price * pool.royalty_for_minting.creator as u64 / 10000 * discount / 100
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.creator_wallet.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }

        if *ctx.accounts.owner.key != parent_nft_owner {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.parent_nft_owner.key,
                    pool.minting_price * pool.royalty_for_minting.parent as u64 / 10000 * discount / 100 
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.parent_nft_owner.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }

        if *ctx.accounts.owner.key != grand_parent_nft_owner {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.grand_parent_nft_owner.key,
                    pool.minting_price * pool.royalty_for_minting.grand_parent as u64 / 10000 * discount / 100 
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.grand_parent_nft_owner.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }

        if *ctx.accounts.owner.key != grand_grand_parent_nft_owner {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.grand_grand_parent_nft_owner.key,
                    pool.minting_price * pool.royalty_for_minting.grand_grand_parent as u64 / 10000 * discount / 100 
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.grand_grand_parent_nft_owner.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }

        msg!("grand grand parent");
        if *ctx.accounts.owner.key != grand_grand_grand_parent_nft_owner {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.owner.key,
                    ctx.accounts.grand_grand_grand_parent_nft_owner.key,
                    pool.minting_price * pool.royalty_for_minting.grand_grand_grand_parent as u64 / 10000 * discount / 100 
                ),
                &[
                    ctx.accounts.owner.clone(),
                    ctx.accounts.grand_grand_grand_parent_nft_owner.clone(),
                    ctx.accounts.system_program.clone(),
                ]
            )?;
        }
        
        msg!("sol transfer done");

        // let mut royalty_scoby = pool.royalty_for_trading.scoby as u8;
        // let mut royalty_creator_scout = pool.royalty_for_trading.creator_scout as u8;
        let mut royalty_creator = pool.royalty_for_trading.creator as u8;
        let mut royalty_parent = pool.royalty_for_trading.parent as u8;
        let mut royalty_grand_parent = pool.royalty_for_trading.grand_parent as u8;
        let mut royalty_grand_grand_parent = pool.royalty_for_trading.grand_grand_parent as u8;
        let mut royalty_grand_grand_grand_parent = pool.royalty_for_trading.grand_grand_grand_parent as u8;
        
        if creator_wallet ==  parent_nft_owner {
            royalty_creator += royalty_parent;
            royalty_parent = 0;
        }
        
        if creator_wallet ==  grand_parent_nft_owner {
            royalty_creator += royalty_grand_parent;
            royalty_grand_parent = 0;
        }

        if creator_wallet ==  grand_grand_parent_nft_owner {
            royalty_creator += royalty_grand_grand_parent;
            royalty_grand_grand_parent = 0;
        }

        if creator_wallet ==  grand_grand_grand_parent_nft_owner {
            royalty_creator += royalty_grand_grand_grand_parent;
            royalty_grand_grand_grand_parent = 0;
        }

        if parent_nft_owner ==  grand_parent_nft_owner {
            royalty_parent += royalty_grand_parent;
            royalty_grand_parent = 0;
        }

        if parent_nft_owner ==  grand_grand_parent_nft_owner {
            royalty_parent += royalty_grand_grand_parent;
            royalty_grand_grand_parent = 0;
        }

        if parent_nft_owner ==  grand_grand_grand_parent_nft_owner {
            royalty_parent += royalty_grand_grand_grand_parent;
            royalty_grand_grand_grand_parent = 0;
        }

        if grand_parent_nft_owner ==  grand_grand_parent_nft_owner {
            royalty_grand_parent += royalty_grand_grand_parent;
            royalty_grand_grand_parent = 0;
        }

        if grand_parent_nft_owner ==  grand_grand_grand_parent_nft_owner {
            royalty_grand_parent += royalty_grand_grand_grand_parent;
            royalty_grand_grand_grand_parent = 0;
        }

        if grand_grand_parent_nft_owner ==  grand_grand_grand_parent_nft_owner {
            royalty_grand_grand_parent += royalty_grand_grand_grand_parent;
            royalty_grand_grand_grand_parent = 0;
        }

        let mut creators : Vec<metaplex_token_metadata::state::Creator> = 
            vec![metaplex_token_metadata::state::Creator{
                address : pool.key(),
                verified : true,
                share : 0,
            }];

        if royalty_creator != 0{
            creators.push(metaplex_token_metadata::state::Creator{
                address : creator_wallet,
                verified : false,
                share : royalty_creator as u8,
            });
        }

        if royalty_parent != 0{
            creators.push(metaplex_token_metadata::state::Creator{
                address : parent_nft_owner,
                verified : false,
                share : royalty_parent as u8,
            });
        }

        if royalty_grand_parent != 0{
            creators.push(metaplex_token_metadata::state::Creator{
                address : grand_parent_nft_owner,
                verified : false,
                share : royalty_grand_parent as u8,
            });
        }

        if royalty_grand_grand_parent != 0{
            creators.push(metaplex_token_metadata::state::Creator{
                address : grand_grand_parent_nft_owner,
                verified : false,
                share : royalty_grand_grand_parent as u8,
            });
        }

        if royalty_grand_grand_grand_parent != 0{
            creators.push(metaplex_token_metadata::state::Creator{
                address : grand_grand_grand_parent_nft_owner,
                verified : false,
                share : royalty_grand_grand_grand_parent as u8,
            });
        }

        // let sparkle_heart = vec![0u8];
        let substring: String ="Hellbenders ".to_owned() + &group_name.to_owned() + &" Spawn #".to_owned() + &group_minting_count.to_string();
        
        // msg!(&config_line.name.clone().replace(std::str::from_utf8(&vec![0u8]).unwrap(), "").len().to_string());
        // msg!(&substring);
        // msg!(&substring.len().to_string());

        let pool_seeds = &[pool.rand.as_ref(),&[pool.bump]];
        invoke_signed(
            &create_metadata_accounts(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.nft_mint.key,
                *ctx.accounts.owner.key,
                *ctx.accounts.owner.key,
                pool.key(),
                substring,
                config_data.symbol.clone(),
                config_line.uri,
                Some(creators),
                config_data.seller_fee,
                true,
                true,
            ),
            &[
                ctx.accounts.metadata.clone(),
                ctx.accounts.nft_mint.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.token_metadata_program.clone(),
                ctx.accounts.token_program.clone(),
                ctx.accounts.system_program.clone(),
                ctx.accounts.rent.to_account_info().clone(),
                pool.to_account_info().clone(),
            ],
            &[pool_seeds],
        )?;
        invoke_signed(
            &create_master_edition(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.master_edition.key,
                *ctx.accounts.nft_mint.key,
                pool.key(),
                *ctx.accounts.owner.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.owner.key,
                None
            ),
            &[
                ctx.accounts.master_edition.clone(),
                ctx.accounts.nft_mint.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.metadata.clone(),
                ctx.accounts.token_metadata_program.clone(),
                ctx.accounts.token_program.clone(),
                ctx.accounts.system_program.clone(),
                ctx.accounts.rent.to_account_info().clone(),
                pool.to_account_info().clone(),
            ],
            &[pool_seeds]
        )?;
        invoke_signed(
            &update_metadata_accounts(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata.key,
                pool.key(),
                Some(pool.update_authority),
                None,
                Some(true),
            ),
            &[
                ctx.accounts.token_metadata_program.clone(),
                ctx.accounts.metadata.clone(),
                pool.to_account_info().clone(),
            ],
            &[pool_seeds]
        )?;
        metadata_extended.mint = nft_mint;
        metadata_extended.minter = *ctx.accounts.owner.key;
        metadata_extended.parent_nfp = parent_nft_account.mint;
        metadata_extended.number = pool.count_minting;
        metadata_extended.bump = _bump;
        pool.count_minting = pool.count_minting + 1;

        if group_name == "Legendary"
        {
            pool.count_group_1 = pool.count_group_1 + 1;
        } else if group_name == "Gold" {
            pool.count_group_2 = pool.count_group_2 + 1;
        } else if group_name == "Steel" {
            pool.count_group_3 = pool.count_group_3 + 1;
        } else {
            pool.count_group_4 = pool.count_group_4 + 1;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct UpdatePool<'info>{
    #[account(mut,signer)]
    owner : AccountInfo<'info>,

    #[account(mut, has_one=owner)]
    pool : ProgramAccount<'info,Pool>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info>{
    #[account(mut)]
    authority : AccountInfo<'info>,

    #[account(mut)]
    config : AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct Mint<'info>{
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(mut)]
    pool : ProgramAccount<'info, Pool>,

    config : AccountInfo<'info>,

    #[account(mut)]
    nft_mint : AccountInfo<'info>,

    #[account(mut)]
    nft_account : AccountInfo<'info>,

    #[account(mut)]
    metadata : AccountInfo<'info>,

    #[account(mut)]
    master_edition : AccountInfo<'info>,

    #[account(init, seeds=[(*nft_mint.key).as_ref(), pool.key().as_ref()], bump=_bump, payer=owner, space=8+METADATA_EXTENDED_SIZE)]
    metadata_extended : ProgramAccount<'info, MetadataExtended>,

    // #[account(mut)]
    // parent_nft_mint : AccountInfo<'info>,

    #[account(mut)]
    parent_nft_account : AccountInfo<'info>,

    #[account(mut)]
    parent_nft_owner : AccountInfo<'info>,

    // #[account(mut)]
    // grand_parent_nft_mint : AccountInfo<'info>,

    // #[account(mut)]
    // grand_parent_nft_account : AccountInfo<'info>,

    #[account(mut)]
    grand_parent_nft_owner : AccountInfo<'info>,

    // #[account(mut)]
    // grand_grand_parent_nft_mint : AccountInfo<'info>,

    // #[account(mut)]
    // grand_grand_parent_nft_account : AccountInfo<'info>,

    #[account(mut)]
    grand_grand_parent_nft_owner : AccountInfo<'info>,

    // #[account(mut)]
    // grand_grand_grand_parent_nft_mint : AccountInfo<'info>,

    // #[account(mut)]
    // grand_grand_grand_parent_nft_account : AccountInfo<'info>,

    #[account(mut)]
    grand_grand_grand_parent_nft_owner : AccountInfo<'info>,

    #[account(mut)]
    creator_nft_account : AccountInfo<'info>,

    #[account(mut)]
    creator_wallet : AccountInfo<'info>,

    // #[account(mut)]
    // redlist_token_account : AccountInfo<'info>,

    // #[account(mut)]
    // creator_scout_nft_account : AccountInfo<'info>,

    // #[account(mut)]
    // creator_scout_wallet : AccountInfo<'info>,

    #[account(mut)]
    scoby_wallet : AccountInfo<'info>,

    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,

    #[account(address = metaplex_token_metadata::id())]
    token_metadata_program: AccountInfo<'info>,

    // system_program : Program<'info,System>,  
    #[account(address = system_program::ID)]
    system_program : AccountInfo<'info>,

    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct MintWithRedlist<'info>{
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(mut)]
    pool : ProgramAccount<'info, Pool>,

    config : AccountInfo<'info>,

    #[account(mut)]
    nft_mint : AccountInfo<'info>,

    #[account(mut)]
    nft_account : AccountInfo<'info>,

    #[account(mut)]
    metadata : AccountInfo<'info>,

    #[account(mut)]
    master_edition : AccountInfo<'info>,

    #[account(init, seeds=[(*nft_mint.key).as_ref(), pool.key().as_ref()], bump=_bump, payer=owner, space=8+METADATA_EXTENDED_SIZE)]
    metadata_extended : ProgramAccount<'info, MetadataExtended>,

    // #[account(mut)]
    // parent_nft_mint : AccountInfo<'info>,

    #[account(mut)]
    parent_nft_account : AccountInfo<'info>,

    #[account(mut)]
    parent_nft_owner : AccountInfo<'info>,

    // #[account(mut)]
    // grand_parent_nft_mint : AccountInfo<'info>,

    // #[account(mut)]
    // grand_parent_nft_account : AccountInfo<'info>,

    #[account(mut)]
    grand_parent_nft_owner : AccountInfo<'info>,

    // #[account(mut)]
    // grand_grand_parent_nft_mint : AccountInfo<'info>,

    // #[account(mut)]
    // grand_grand_parent_nft_account : AccountInfo<'info>,

    #[account(mut)]
    grand_grand_parent_nft_owner : AccountInfo<'info>,

    // #[account(mut)]
    // grand_grand_grand_parent_nft_mint : AccountInfo<'info>,

    // #[account(mut)]
    // grand_grand_grand_parent_nft_account : AccountInfo<'info>,

    #[account(mut)]
    grand_grand_grand_parent_nft_owner : AccountInfo<'info>,

    #[account(mut)]
    creator_nft_account : AccountInfo<'info>,

    #[account(mut)]
    creator_wallet : AccountInfo<'info>,

    #[account(mut)]
    redlist_token_account : AccountInfo<'info>,

    // #[account(mut)]
    // redlist_token_account : AccountInfo<'info>,

    // #[account(mut)]
    // creator_scout_nft_account : AccountInfo<'info>,

    // #[account(mut)]
    // creator_scout_wallet : AccountInfo<'info>,

    #[account(mut)]
    scoby_wallet : AccountInfo<'info>,

    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,

    #[account(address = metaplex_token_metadata::id())]
    token_metadata_program: AccountInfo<'info>,

    // system_program : Program<'info,System>,  
    #[account(address = system_program::ID)]
    system_program : AccountInfo<'info>,

    rent: Sysvar<'info, Rent>,
}




#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct InitPool<'info>{
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(init, seeds=[(*rand.key).as_ref()], bump=_bump, payer=owner, space=8+POOL_SIZE)]
    pool : ProgramAccount<'info, Pool>,

    rand : AccountInfo<'info>,

    config : AccountInfo<'info>,

    system_program : Program<'info,System>,  
}

#[derive(Accounts)]
pub struct AddConfigLines<'info> {
    #[account(mut, signer)]
    authority : AccountInfo<'info>,

    // #[account(mut, has_one=authority)]
    // config : ProgramAccount<'info, Config>,
    #[account(mut)]
    config : AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(_max_number_of_lines : u32)]
pub struct InitConfig<'info>{
    #[account(mut, signer)]
    authority : AccountInfo<'info>,

    #[account(mut, constraint= config.to_account_info().owner==program_id && config.to_account_info().data_len() >= CONFIG_SIZE+(4+CONFIG_LINE_SIZE * _max_number_of_lines as usize))]
    config : AccountInfo<'info>,
}

pub const METADATA_EXTENDED_SIZE : usize = 32 + 32 + 32 + 4 + 1;
#[account]
pub struct MetadataExtended{
    pub mint : Pubkey,
    pub minter : Pubkey,
    pub parent_nfp : Pubkey,
    pub number : u32,
    pub bump : u8,
}

#[account]
pub struct Metadata{
    pub name : String,
    // pub symbol : String,
    pub uri : String,
    // pub seller_fee_basis_points : u16,
    // pub creators : Vec<Creator>,
    // pub is_mutable : bool,
}


// pub const POOL_SIZE : usize = 32 + 32 + 32 + 4 + 8 + 32 + 32 + ROYALTY_SIZE + ROYALTY_SIZE + 32 + 1;
pub const POOL_SIZE : usize = 32 + 32 + 32 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 8 + 32 + 32 + 32 + ROYALTY_SIZE + ROYALTY_SIZE + 32 + 32 + 32 + 32 + 32 + 1;
#[account]
#[derive(Default)]
pub struct Pool{
    pub owner : Pubkey,
    pub rand : Pubkey,
    pub config : Pubkey,
    pub count_group_1 : u32,
    pub count_group_2 : u32,
    pub count_group_3 : u32,
    pub count_group_4 : u32,
    pub count_group_5 : u32,
    pub count_group_6 : u32,
    pub count_minting : u32,
    pub minting_price : u64,
    pub update_authority : Pubkey,
    pub root_nft : Pubkey,
    pub creator_scout : Pubkey,
    pub royalty_for_minting : Royalty,
    pub royalty_for_trading : Royalty,
    pub scoby_wallet : Pubkey,
    pub legendary : Pubkey,
    pub redlist_black : Pubkey,
    pub redlist_steel : Pubkey,
    pub redlist_gold : Pubkey,
    pub bump : u8
}

pub const ROYALTY_SIZE : usize = 2 + 2 + 2 + 2 + 2 + 2;
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Royalty{
    pub scoby : u16,
    pub creator : u16,
    pub parent : u16,
    pub grand_parent : u16,
    pub grand_grand_parent : u16,
    pub grand_grand_grand_parent : u16
}

pub const CONFIG_SIZE : usize = 32 + 4 + CONFIG_DATA_SIZE; // + 4 + CONFIG_LINE_SIZE * max_number_of_lines
#[account]
#[derive(Default)]
pub struct Config{
    pub authority : Pubkey,
    pub max_number_of_lines : u32,
    pub config_data : ConfigData,
    pub config_lines : Vec<ConfigLine>
}

pub const CONFIG_DATA_SIZE : usize = 4 + MAX_SYMBOL_LENGTH + 32 + 2;
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ConfigData{
    pub symbol : String,
    pub creator : Pubkey,
    pub seller_fee : u16,
}

pub const CONFIG_LINE_SIZE : usize = 4 + MAX_NAME_LENGTH + 4 + MAX_URI_LENGTH;
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ConfigLine{
    pub name : String,
    pub uri : String,
}

pub fn get_authority(
    a : &AccountInfo,
    ) -> core::result::Result<Pubkey,ProgramError> {
    let arr = a.data.borrow();
    let data_array = &arr[8..40];
    let authority : Pubkey = Pubkey::try_from_slice(data_array)?;
    Ok(authority)
}

pub fn get_config_data(
    a : &AccountInfo,
    ) -> core::result::Result<ConfigData,ProgramError> {
    let arr = a.data.borrow();
    let data_array = &arr[8+32+4..8+32+4+CONFIG_DATA_SIZE];
    let config_data : ConfigData = ConfigData::try_from_slice(data_array)?;
    Ok(config_data)
}

pub fn get_config_count(data : &Ref<&mut [u8]>) -> core::result::Result<usize, ProgramError>{
    return Ok(u32::from_le_bytes(*array_ref![data, 8+CONFIG_SIZE, 4]) as usize);
}

pub fn set_config_count(a : &mut AccountInfo, count : u32){
    let mut arr = a.data.borrow_mut();
    let data_array = count.try_to_vec().unwrap();
    let vec_start = 8 + CONFIG_SIZE;
    for i in 0..data_array.len() {
        arr[vec_start+i] = data_array[i];
    }
}

pub fn get_config_line(
    a : &AccountInfo,
    index : usize,
    ) -> core::result::Result<ConfigLine, ProgramError> {
    let arr = a.data.borrow();
    let total = get_config_count(&arr)?;
    if index > total {
        return Err(PoolError::IndexGreaterThanLength.into());
    }
    let data_array = &arr[8+CONFIG_SIZE + 4 + index * (CONFIG_LINE_SIZE)..8+CONFIG_SIZE + 4 + (index+1) * (CONFIG_LINE_SIZE)];
    let config_line : ConfigLine = ConfigLine::try_from_slice(data_array)?;
    Ok(config_line)
}

#[error]
pub enum PoolError {
    #[msg("Token mint to failed")]
    TokenMintToFailed,

    #[msg("Token set authority failed")]
    TokenSetAuthorityFailed,

    #[msg("Token transfer failed")]
    TokenTransferFailed,

    #[msg("Invalid mint account")]
    InvalidMintAccount,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Invalid pool account")]
    InvalidPoolAccount,

    #[msg("Mint amount is zero")]
    MintAmountIsZero,

    #[msg("Index greater than length")]
    IndexGreaterThanLength,

    #[msg("Not enough sol")]
    NotEnoughSol,

    #[msg("Invalid mint pre requirement")]
    InvalidMintPrerequirement,

    #[msg("Invalid oldest mint requirement")]
    InvalidOldestMintRequirement,

    #[msg("Invalid oldest metadata_extended")]
    InvalidOldestMetadataExtended,

    #[msg("Invalid parent")]
    InvalidParent,

    #[msg("Invalid pool wallet")]
    InvalidPoolWallet,

    #[msg("Invalid creator wallet")]
    InvalidCreatorWallet,

    #[msg("Invalid creator scout account")]
    InvalidCreatorScoutRequirement,

    #[msg("Invalid creating root")]
    InvalidCreatingRoot,

    #[msg("Invalid authority")]
    InvalidAuthority,

    #[msg("Invalid owner")]
    InvalidOwner,

    #[msg("Invalid index")]
    InvalidIndex,
}