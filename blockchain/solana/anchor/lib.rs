#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

pub mod dot;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use dot::program::*;
use std::{cell::RefCell, rc::Rc};

declare_id!("BgXa2B5G45eCm7WffVPCj5hVhy3kV9cgfWafpFdiW6m4");

pub mod seahorse_util {
    use super::*;

    #[cfg(feature = "pyth-sdk-solana")]
    pub use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};
    use std::{collections::HashMap, fmt::Debug, ops::Deref};

    pub struct Mutable<T>(Rc<RefCell<T>>);

    impl<T> Mutable<T> {
        pub fn new(obj: T) -> Self {
            Self(Rc::new(RefCell::new(obj)))
        }
    }

    impl<T> Clone for Mutable<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Deref for Mutable<T> {
        type Target = Rc<RefCell<T>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Debug> Debug for Mutable<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl<T: Default> Default for Mutable<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    impl<T: Clone> Mutable<Vec<T>> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    impl<T: Clone, const N: usize> Mutable<[T; N]> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    #[derive(Clone)]
    pub struct Empty<T: Clone> {
        pub account: T,
        pub bump: Option<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct ProgramsMap<'info>(pub HashMap<&'static str, AccountInfo<'info>>);

    impl<'info> ProgramsMap<'info> {
        pub fn get(&self, name: &'static str) -> AccountInfo<'info> {
            self.0.get(name).unwrap().clone()
        }
    }

    #[derive(Clone, Debug)]
    pub struct WithPrograms<'info, 'entrypoint, A> {
        pub account: &'entrypoint A,
        pub programs: &'entrypoint ProgramsMap<'info>,
    }

    impl<'info, 'entrypoint, A> Deref for WithPrograms<'info, 'entrypoint, A> {
        type Target = A;

        fn deref(&self) -> &Self::Target {
            &self.account
        }
    }

    pub type SeahorseAccount<'info, 'entrypoint, A> =
        WithPrograms<'info, 'entrypoint, Box<Account<'info, A>>>;

    pub type SeahorseSigner<'info, 'entrypoint> = WithPrograms<'info, 'entrypoint, Signer<'info>>;

    #[derive(Clone, Debug)]
    pub struct CpiAccount<'info> {
        #[doc = "CHECK: CpiAccounts temporarily store AccountInfos."]
        pub account_info: AccountInfo<'info>,
        pub is_writable: bool,
        pub is_signer: bool,
        pub seeds: Option<Vec<Vec<u8>>>,
    }

    #[macro_export]
    macro_rules! seahorse_const {
        ($ name : ident , $ value : expr) => {
            macro_rules! $name {
                () => {
                    $value
                };
            }

            pub(crate) use $name;
        };
    }

    #[macro_export]
    macro_rules! assign {
        ($ lval : expr , $ rval : expr) => {{
            let temp = $rval;

            $lval = temp;
        }};
    }

    #[macro_export]
    macro_rules! index_assign {
        ($ lval : expr , $ idx : expr , $ rval : expr) => {
            let temp_rval = $rval;
            let temp_idx = $idx;

            $lval[temp_idx] = temp_rval;
        };
    }

    pub(crate) use assign;

    pub(crate) use index_assign;

    pub(crate) use seahorse_const;
}

#[program]
mod electra_chain {
    use super::*;
    use seahorse_util::*;
    use std::collections::HashMap;

    #[derive(Accounts)]
    # [instruction (secret_key_array: [u8; 32] , time_to_hide : u32)]
    pub struct HideParkingArea<'info> {
        #[account()]
        pub clock: Sysvar<'info, Clock>,
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        pub user: Signer<'info>,
        #[account(mut)]
        pub parking_area: Box<Account<'info, dot::program::ParkingArea>>,
        pub system_program: Program<'info, System>,
    }

    pub fn hide_parking_area(
        ctx: Context<HideParkingArea>,
        secret_key_array: [u8; 32],
        time_to_hide: u32,
    ) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let clock = &ctx.accounts.clock.clone();
        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let user = SeahorseSigner {
            account: &ctx.accounts.user,
            programs: &programs_map,
        };

        let parking_area =
            dot::program::ParkingArea::load(&mut ctx.accounts.parking_area, &programs_map);

        hide_parking_area_handler(
            clock.clone(),
            payer.clone(),
            user.clone(),
            parking_area.clone(),
            secret_key_array,
            time_to_hide,
        );

        dot::program::ParkingArea::store(parking_area);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (coordinates_class: Coordinates , name_array: [u16; 32] , address_array: [u16; 64] , info_array: [u16; 256] , price : u32 , secret_key_array: [u8; 32] , seed_random : u64)]
    pub struct InitParkingArea<'info> {
        #[account()]
        pub clock: Sysvar<'info, Clock>,
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        pub owner: Signer<'info>,
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: ParkingArea > () + 8 , payer = payer , seeds = [owner . key () . as_ref () , "parking_area" . as_bytes () . as_ref () , seed_random . to_le_bytes () . as_ref ()] , bump)]
        pub parking_area: Box<Account<'info, dot::program::ParkingArea>>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
    }

    pub fn init_parking_area(
        ctx: Context<InitParkingArea>,
        coordinates_class: Coordinates,
        name_array: [u16; 32],
        address_array: [u16; 64],
        info_array: [u16; 256],
        price: u32,
        secret_key_array: [u8; 32],
        seed_random: u64,
    ) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let clock = &ctx.accounts.clock.clone();
        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let owner = SeahorseSigner {
            account: &ctx.accounts.owner,
            programs: &programs_map,
        };

        let parking_area = Empty {
            account: dot::program::ParkingArea::load(&mut ctx.accounts.parking_area, &programs_map),
            bump: Some(ctx.bumps.parking_area),
        };

        init_parking_area_handler(
            clock.clone(),
            payer.clone(),
            owner.clone(),
            parking_area.clone(),
            coordinates_class,
            name_array,
            address_array,
            info_array,
            price,
            secret_key_array,
            seed_random,
        );

        dot::program::ParkingArea::store(parking_area.account);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (address_array: [u16; 64] , info_array: [u16; 256] , price : u32 , secret_key_array: [u8; 32])]
    pub struct UpdateParkingArea<'info> {
        #[account()]
        pub clock: Sysvar<'info, Clock>,
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        pub owner: Signer<'info>,
        #[account(mut)]
        pub parking_area: Box<Account<'info, dot::program::ParkingArea>>,
    }

    pub fn update_parking_area(
        ctx: Context<UpdateParkingArea>,
        address_array: [u16; 64],
        info_array: [u16; 256],
        price: u32,
        secret_key_array: [u8; 32],
    ) -> Result<()> {
        let mut programs = HashMap::new();
        let programs_map = ProgramsMap(programs);
        let clock = &ctx.accounts.clock.clone();
        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let owner = SeahorseSigner {
            account: &ctx.accounts.owner,
            programs: &programs_map,
        };

        let parking_area =
            dot::program::ParkingArea::load(&mut ctx.accounts.parking_area, &programs_map);

        update_parking_area_handler(
            clock.clone(),
            payer.clone(),
            owner.clone(),
            parking_area.clone(),
            address_array,
            info_array,
            price,
            secret_key_array,
        );

        dot::program::ParkingArea::store(parking_area);

        return Ok(());
    }
}
