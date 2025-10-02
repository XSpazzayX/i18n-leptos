use leptos::prelude::*;

pub mod local_storage {
    use super::*;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum LocalStorageError {
        #[error("local storage is not available")]
        NotAvailable,
        #[error("failed to set item in local storage")]
        SetError,
        #[error("failed to get item from local storage")]
        GetError,
    }

    pub fn set(key: &str, value: &str) -> Result<(), LocalStorageError> {
        window()
            .local_storage()
            .map_err(|_| LocalStorageError::NotAvailable)?
            .ok_or(LocalStorageError::NotAvailable)?
            .set_item(key, value)
            .map_err(|_| LocalStorageError::SetError)
    }

    pub fn get(key: &str) -> Result<Option<String>, LocalStorageError> {
        window()
            .local_storage()
            .map_err(|_| LocalStorageError::NotAvailable)?
            .ok_or(LocalStorageError::NotAvailable)?
            .get_item(key)
            .map_err(|_| LocalStorageError::GetError)
    }
}
