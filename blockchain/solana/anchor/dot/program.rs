#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use crate::{id, seahorse_util::*};
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct Coordinates {
    pub lat: f64,
    pub long: f64,
}

#[account]
#[derive(Debug)]
pub struct ParkingArea {
    pub owner: Pubkey,
    pub user: Pubkey,
    pub coordinates_class: Coordinates,
    pub name_array: [u16; 32],
    pub address_array: [u16; 64],
    pub info_array: [u16; 256],
    pub price: u32,
    pub expired_time: i64,
    pub secret_key_array: [u8; 32],
}

impl<'info, 'entrypoint> ParkingArea {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedParkingArea<'info, 'entrypoint>> {
        let owner = account.owner.clone();
        let user = account.user.clone();
        let coordinates_class =
            Mutable::new(account.coordinates_class.clone());

        let name_array = Mutable::new(account.name_array.clone());
        let address_array = Mutable::new(account.address_array.clone());
        let info_array = Mutable::new(account.info_array.clone());
        let price = account.price;
        let expired_time = account.expired_time;
        let secret_key_array = Mutable::new(account.secret_key_array.clone());

        Mutable::new(LoadedParkingArea {
            __account__: account,
            __programs__: programs_map,
            owner,
            user,
            coordinates_class,
            name_array,
            address_array,
            info_array,
            price,
            expired_time,
            secret_key_array,
        })
    }

    pub fn store(loaded: Mutable<LoadedParkingArea>) {
        let mut loaded = loaded.borrow_mut();
        let owner = loaded.owner.clone();

        loaded.__account__.owner = owner;

        let user = loaded.user.clone();

        loaded.__account__.user = user;

        let coordinates_class = loaded.coordinates_class.borrow().clone();

        loaded.__account__.coordinates_class = coordinates_class;

        let name_array = loaded.name_array.borrow().clone();

        loaded.__account__.name_array = name_array;

        let address_array = loaded.address_array.borrow().clone();

        loaded.__account__.address_array = address_array;

        let info_array = loaded.info_array.borrow().clone();

        loaded.__account__.info_array = info_array;

        let price = loaded.price;

        loaded.__account__.price = price;

        let expired_time = loaded.expired_time;

        loaded.__account__.expired_time = expired_time;

        let secret_key_array = loaded.secret_key_array.borrow().clone();

        loaded.__account__.secret_key_array = secret_key_array;
    }
}

#[derive(Debug)]
pub struct LoadedParkingArea<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, ParkingArea>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub owner: Pubkey,
    pub user: Pubkey,
    pub coordinates_class: Mutable<Coordinates>,
    pub name_array: Mutable<[u16; 32]>,
    pub address_array: Mutable<[u16; 64]>,
    pub info_array: Mutable<[u16; 256]>,
    pub price: u32,
    pub expired_time: i64,
    pub secret_key_array: Mutable<[u8; 32]>,
}

pub fn hide_parking_area_handler<'info>(
    mut clock: Sysvar<'info, Clock>,
    mut payer: SeahorseSigner<'info, '_>,
    mut user: SeahorseSigner<'info, '_>,
    mut parking_area: Mutable<LoadedParkingArea<'info, '_>>,
    mut secret_key_array: [u8; 32],
    mut time_to_hide: u32,
) -> () {
    let mut time = clock.unix_timestamp;

    if !(time_to_hide > 0) {
        panic!("The time to hide is not valid");
    }

    if parking_area.borrow().user != user.key() {
        if !(parking_area.borrow().expired_time <= time) {
            panic!("The parking area is not expired");
        }
    }

    let mut is_secret_key_ok = true;
    let mut secret_key_mut_array = Mutable::<[u8; 32]>::new(secret_key_array);

    for mut i in 0..(secret_key_mut_array.borrow().len() as u64) {
        if secret_key_mut_array.borrow()
            [secret_key_mut_array.wrapped_index((i as i128) as i128)]
            != parking_area.borrow().secret_key_array.borrow()[parking_area
                .borrow()
                .secret_key_array
                .wrapped_index((i as i128) as i128)]
        {
            is_secret_key_ok = false;

            break;
        }
    }

    if !is_secret_key_ok {
        panic!("The secret key is not valid");
    }

    solana_program::program::invoke(
        &solana_program::system_instruction::transfer(
            &user.key(),
            &parking_area.borrow().__account__.key(),
            <u64 as TryFrom<_>>::try_from((parking_area.borrow().price * time_to_hide)).unwrap(),
        ),
        &[
            user.to_account_info(),
            parking_area.borrow().__account__.to_account_info(),
            user.programs.get("system_program").clone(),
        ],
    )
    .unwrap();

    if parking_area.borrow().user != user.key() {
        assign!(parking_area.borrow_mut().user, user.key());
    }

    if parking_area.borrow().expired_time > time {
        assign!(
            parking_area.borrow_mut().expired_time,
            parking_area.borrow().expired_time + (time_to_hide as i64)
        );
    } else {
        assign!(
            parking_area.borrow_mut().expired_time,
            time + (time_to_hide as i64)
        );
    }
}

pub fn init_parking_area_handler<'info>(
    mut clock: Sysvar<'info, Clock>,
    mut payer: SeahorseSigner<'info, '_>,
    mut owner: SeahorseSigner<'info, '_>,
    mut parking_area: Empty<Mutable<LoadedParkingArea<'info, '_>>>,
    mut coordinates_class: Coordinates,
    mut name_array: [u16; 32],
    mut address_array: [u16; 64],
    mut info_array: [u16; 256],
    mut price: u32,
    mut secret_key_array: [u8; 32],
    mut seed_random: u64,
) -> () {
    let mut time = clock.unix_timestamp;
    let mut parking_area = parking_area.account.clone();

    assign!(parking_area.borrow_mut().owner, owner.key());

    assign!(parking_area.borrow_mut().coordinates_class, Mutable::<Coordinates>::new(coordinates_class));

    assign!(parking_area.borrow_mut().name_array, Mutable::<[u16; 32]>::new(name_array));

    assign!(parking_area.borrow_mut().address_array, Mutable::<[u16; 64]>::new(address_array));

    assign!(parking_area.borrow_mut().info_array, Mutable::<[u16; 256]>::new(info_array));

    assign!(parking_area.borrow_mut().price, price);

    assign!(parking_area.borrow_mut().expired_time, time);

    assign!(parking_area.borrow_mut().secret_key_array, Mutable::<[u8; 32]>::new(secret_key_array));
}

pub fn update_parking_area_handler<'info>(
    mut clock: Sysvar<'info, Clock>,
    mut payer: SeahorseSigner<'info, '_>,
    mut owner: SeahorseSigner<'info, '_>,
    mut parking_area: Mutable<LoadedParkingArea<'info, '_>>,
    mut address_array: [u16; 64],
    mut info_array: [u16; 256],
    mut price: u32,
    mut secret_key_array: [u8; 32],
) -> () {
    let mut time = clock.unix_timestamp;

    if !(parking_area.borrow().owner == owner.key()) {
        panic!("The owner is not the same");
    }

    assign!(parking_area.borrow_mut().address_array, Mutable::<[u16; 64]>::new(address_array));

    assign!(parking_area.borrow_mut().info_array, Mutable::<[u16; 256]>::new(info_array));

    assign!(parking_area.borrow_mut().price, price);

    assign!(parking_area.borrow_mut().expired_time, time);

    assign!(parking_area.borrow_mut().secret_key_array, Mutable::<[u8; 32]>::new(secret_key_array));
}
