use crate::{
    constants::REVEL_SESSION,
    cookie::{self, storage::CookieStorage},
};

pub fn get_revel_session() -> Option<String> {
    let cookie_path = cookie::utils::get_cookie_path();
    let storage = CookieStorage::new(&cookie_path);

    let cookie = storage.get(REVEL_SESSION).ok().flatten();

    match &cookie {
        Some(_) => {
            let path_str = cookie_path.display().to_string();
            success_ln!("Using Cookie in {}", path_str);
        }
        None => {
            warn_ln!("Cookie is not set.");
        }
    };

    cookie
}


