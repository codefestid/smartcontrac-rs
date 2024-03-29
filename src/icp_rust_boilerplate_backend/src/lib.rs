#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

const COUNTER_MEMORY_ID: MemoryId = MemoryId::new(0);
const RENTAL_STORAGE_MEMORY_ID: MemoryId = MemoryId::new(1);

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Rental {
    id: u64,
    motorcycle_brand: String,
    daily_rate: u64,
    rental_date: String,
    renter_name: String,
    rental_days: u64,
}

impl Storable for Rental {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Rental {
    const MAX_SIZE: u32 = 4096;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(COUNTER_MEMORY_ID)), 0)
            .expect("Cannot create a counter")
    );

    static RENTAL_STORAGE: RefCell<StableBTreeMap<u64, Rental, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(RENTAL_STORAGE_MEMORY_ID)))
    );
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct RentalInput {
    motorcycle_brand: String,
    daily_rate: u64,
    rental_date: String,
    renter_name: String,
    rental_days: u64,
}

#[ic_cdk::query]
fn get_rental(id: u64) -> Result<Rental, Error> {
    _get_rental(&id).ok_or_else(|| Error::NotFound {
        msg: format!("Rental with id={} not found", id),
    })
}

#[ic_cdk::update]
fn add_rental(input: RentalInput) -> Result<Rental, Error> {
    // Ensure input fields are not empty or invalid
    if input.motorcycle_brand.is_empty() || input.rental_date.is_empty() || input.renter_name.is_empty() || input.rental_days == 0 {
        return Err(Error::InvalidInput {
            msg: "Some fields are empty or invalid".to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let rental = Rental {
        id,
        motorcycle_brand: input.motorcycle_brand,
        daily_rate: input.daily_rate,
        rental_date: input.rental_date,
        renter_name: input.renter_name,
        rental_days: input.rental_days,
    };

    do_insert_study_group(&rental);
    Ok(rental)
}

// Helper method to insert study group into storage
fn do_insert_study_group(rental: &Rental) {
    RENTAL_STORAGE.with(|service| service.borrow_mut().insert(rental.id, rental.clone()));
}


#[ic_cdk::update]
fn update_rental(id: u64, input: RentalInput) -> Result<Rental, Error> {
    match _get_rental(&id) {
        Some(mut rental) => {
            // Ensure input fields are not empty or invalid
            if input.motorcycle_brand.is_empty() || input.rental_date.is_empty() || input.renter_name.is_empty() || input.rental_days == 0 {
                return Err(Error::InvalidInput {
                    msg: "Some fields are empty or invalid".to_string(),
                });
            }

            // Update the rental with new values
            rental.motorcycle_brand = input.motorcycle_brand;
            rental.daily_rate = input.daily_rate;
            rental.rental_date = input.rental_date;
            rental.renter_name = input.renter_name;
            rental.rental_days = input.rental_days;
            // Update the rental's updated_at timestamp
            rental.updated_at = Some(time());

            // Update the rental in storage
            RENTAL_STORAGE.with(|service| service.borrow_mut().insert(id, rental.clone()));

            Ok(rental)
        }
        None => Err(Error::NotFound {
            msg: format!("Rental with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn delete_rental(id: u64) -> Result<Rental, Error> {
    RENTAL_STORAGE.with(|service| {
        service
            .borrow_mut()
            .remove(&id)
            .ok_or_else(|| Error::NotFound {
                msg: format!("Rental with id={} not found", id),
            })
    })
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidInput { msg: String }
}

// Helper method to get a study group by ID
fn _get_rental(id: &u64) -> Option<Rental> {
    RENTAL_STORAGE.with(|service| service.borrow().get(id).clone())
}

// Need this to generate candid.
ic_cdk::export_candid!();
