#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Define a structure to hold language learning content
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct LanguageLearningContent {
    id: u64,
    text: String,
    image_url: String,
    sound_url: String,
    scent_description: String,
    created_at: u64,
    updated_at: Option<u64>,
}

impl Storable for LanguageLearningContent {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for LanguageLearningContent {
    const MAX_SIZE: u32 = 2048; // Increased size to accommodate additional data
    const IS_FIXED_SIZE: bool = false;
}

// Thread-local storage for MemoryManager, ID counter, and Language Learning content storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static LANGUAGE_CONTENT_STORAGE: RefCell<StableBTreeMap<u64, LanguageLearningContent, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

}

// Define a payload structure for adding language learning content
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct LanguageLearningContentPayload {
    text: String,
    image_url: String,
    sound_url: String,
    scent_description: String,
}

// Add function to retrieve language learning content by ID
#[ic_cdk::query]
fn get_language_content(id: u64) -> Result<LanguageLearningContent, Error> {
    match _get_language_content(&id) {
        Some(content) => Ok(content),
        None => Err(Error::NotFound {
            msg: format!("Language learning content with id={} not found", id),
        }),
    }
}

// Add function to add new language learning content
#[ic_cdk::update]
fn add_language_content(
    content: LanguageLearningContentPayload,
) -> Result<LanguageLearningContent, Error> {
    // Ensure input fields are not empty
    if content.text.is_empty()
        || content.image_url.is_empty()
        || content.sound_url.is_empty()
        || content.scent_description.is_empty()
    {
        return Err(Error::InvalidInput {
            msg: "Text, image URL, sound URL, or scent description cannot be empty".to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let new_content = LanguageLearningContent {
        id,
        text: content.text,
        image_url: content.image_url,
        sound_url: content.sound_url,
        scent_description: content.scent_description,
        created_at: time(),
        updated_at: None,
    };

    do_insert_language_content(&new_content);
    Ok(new_content)
}

// Helper method to insert language learning content into storage
fn do_insert_language_content(content: &LanguageLearningContent) {
    LANGUAGE_CONTENT_STORAGE
        .with(|service| service.borrow_mut().insert(content.id, content.clone()));
}

// Add function to update existing language learning content
#[ic_cdk::update]
fn update_language_content(
    id: u64,
    content: LanguageLearningContentPayload,
) -> Result<LanguageLearningContent, Error> {
    // Ensure input fields are not empty
    if content.text.is_empty()
        || content.image_url.is_empty()
        || content.sound_url.is_empty()
        || content.scent_description.is_empty()
    {
        return Err(Error::InvalidInput {
            msg: "Text, image URL, sound URL, or scent description cannot be empty".to_string(),
        });
    }

    match LANGUAGE_CONTENT_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut content) => {
            content.text = content.text;
            content.image_url = content.image_url;
            content.sound_url = content.sound_url;
            content.scent_description = content.scent_description;
            content.updated_at = Some(time());
            do_insert_language_content(&content);
            Ok(content)
        }
        None => Err(Error::NotFound {
            msg: format!("Language learning content with id={} not found", id),
        }),
    }
}

// Add function to delete language learning content
#[ic_cdk::update]
fn delete_language_content(id: u64) -> Result<LanguageLearningContent, Error> {
    match LANGUAGE_CONTENT_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(content) => Ok(content),
        None => Err(Error::NotFound {
            msg: format!("Language learning content with id={} not found", id),
        }),
    }
}

// Define an enum for error handling
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidInput { msg: String }, // Adding the InvalidInput variant
}

// Helper method to get language learning content by id
fn _get_language_content(id: &u64) -> Option<LanguageLearningContent> {
    LANGUAGE_CONTENT_STORAGE.with(|service| service.borrow().get(id))
}

// Function to list all language learning content
#[ic_cdk::query]
fn list_language_content() -> Vec<LanguageLearningContent> {
    LANGUAGE_CONTENT_STORAGE.with(|service| {
        service
            .borrow()
            .iter()
            .map(|(_, content)| content.clone())
            .collect()
    })
}

// Function to search language learning content by text
#[ic_cdk::query]
fn search_language_content_by_text(keyword: String) -> Vec<LanguageLearningContent> {
    LANGUAGE_CONTENT_STORAGE.with(|service| {
        service
            .borrow()
            .iter()
            .filter(|(_, content)| content.text.contains(&keyword))
            .map(|(_, content)| content.clone())
            .collect()
    })
}

// Function to filter language learning content by scent description
#[ic_cdk::query]
fn filter_language_content_by_scent(keyword: String) -> Vec<LanguageLearningContent> {
    LANGUAGE_CONTENT_STORAGE.with(|service| {
        service
            .borrow()
            .iter()
            .filter(|(_, content)| content.scent_description.contains(&keyword))
            .map(|(_, content)| content.clone())
            .collect()
    })
}

// Function to sort language learning content by creation date
#[ic_cdk::query]
fn sort_language_content_by_creation_date() -> Vec<LanguageLearningContent> {
    let mut content: Vec<LanguageLearningContent> = LANGUAGE_CONTENT_STORAGE.with(|service| {
        service
            .borrow()
            .iter()
            .map(|(_, content)| content.clone())
            .collect()
    });

    content.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    content
}

// Define a structure to hold study group information
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct StudyGroup {
    id: u64,
    name: String,
    members: Vec<String>,
}

impl Storable for StudyGroup {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for StudyGroup {
    const MAX_SIZE: u32 = 2048; // Increased size to accommodate additional data
    const IS_FIXED_SIZE: bool = false;
}

// Thread-local storage for Study Group storage
thread_local! {
    static STUDY_GROUPS: RefCell<StableBTreeMap<u64, StudyGroup, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));
}

// Define a payload structure for creating a study group
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct StudyGroupPayload {
    name: String,
    members: Vec<String>,
}

// Add function to create a new study group with the provided payload
#[ic_cdk::update]
fn create_study_group(payload: StudyGroupPayload) -> Result<StudyGroup, Error> {
    // Ensure the name field is not empty
    if payload.name.is_empty() {
        return Err(Error::InvalidInput {
            msg: "Study group name cannot be empty".to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let new_group = StudyGroup {
        id,
        name: payload.name,
        members: payload.members,
    };

    do_insert_study_group(&new_group);
    Ok(new_group)
}

// Helper method to insert study group into storage
fn do_insert_study_group(group: &StudyGroup) {
    STUDY_GROUPS.with(|service| service.borrow_mut().insert(group.id, group.clone()));
}

// Add function to retrieve a study group by ID
#[ic_cdk::query]
fn get_study_group(id: u64) -> Result<StudyGroup, Error> {
    match _get_study_group(&id) {
        Some(group) => Ok(group),
        None => Err(Error::NotFound {
            msg: format!("Study group with id={} not found", id),
        }),
    }
}

// Helper method to get a study group by ID
fn _get_study_group(id: &u64) -> Option<StudyGroup> {
    STUDY_GROUPS.with(|service| service.borrow().get(id).clone())
}

// Add function to update an existing study group
#[ic_cdk::update]
fn update_study_group(id: u64, payload: StudyGroupPayload) -> Result<StudyGroup, Error> {
    // Ensure the name field is not empty
    if payload.name.is_empty() {
        return Err(Error::InvalidInput {
            msg: "Study group name cannot be empty".to_string(),
        });
    }

    match STUDY_GROUPS.with(|service| service.borrow().get(&id)) {
        Some(mut group) => {
            group.name = payload.name;
            group.members = payload.members;
            do_insert_study_group(&group);
            Ok(group)
        }
        None => Err(Error::NotFound {
            msg: format!("Study group with id={} not found", id),
        }),
    }
}

// Add function to delete a study group
#[ic_cdk::update]
fn delete_study_group(id: u64) -> Result<StudyGroup, Error> {
    match STUDY_GROUPS.with(|service| service.borrow_mut().remove(&id)) {
        Some(group) => Ok(group),
        None => Err(Error::NotFound {
            msg: format!("Study group with id={} not found", id),
        }),
    }
}

// Function to list all study groups
#[ic_cdk::query]
fn list_study_groups() -> Vec<StudyGroup> {
    STUDY_GROUPS.with(|service| {
        service
            .borrow()
            .iter()
            .map(|(_, group)| group.clone())
            .collect()
    })
}

// Export Candid interface
ic_cdk::export_candid!();
