use bevy::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HashPart {
    pub foreign_key: Option<String>,
    pub page_key: String,
    pub tab_num: u16,
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct UrlHash {
    pub hash_parts: Vec<HashPart>,
}

impl UrlHash {
    pub fn parse(raw: &str) -> Self {
        let raw = raw.strip_prefix('#').unwrap_or(raw);

        let hash_parts = raw
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(HashPart::parse)
            .collect();

        UrlHash { hash_parts }
    }
}

impl HashPart {
    fn parse(segment: &str) -> Self {
        let fields: Vec<&str> = segment.split('.').collect();

        match fields.as_slice() {
            [foreign_key, page_key, tab_num] => HashPart {
                foreign_key: Some((*foreign_key).to_string()),
                page_key: (*page_key).to_string(),
                tab_num: tab_num.parse().unwrap_or(0),
            },
            [page_key, tab_num] => HashPart {
                foreign_key: None,
                page_key: (*page_key).to_string(),
                tab_num: tab_num.parse().unwrap_or(0),
            },
            [page_key] => HashPart {
                foreign_key: None,
                page_key: (*page_key).to_string(),
                tab_num: 0,
            },
            _ => HashPart::default(),
        }
    }
}

/// Plugin accepts an optional initial hash string — used to seed `UrlHash`
/// on startup. On native, this comes from a CLI argument (there's no
/// browser URL to read); on wasm, the real hashchange listener takes over
/// and the initial value gets overwritten by whatever the actual page URL
/// contains almost immediately.
pub struct UrlListenerPlugin {
    pub initial_hash: Option<String>,
}

impl Plugin for UrlListenerPlugin {
    fn build(&self, app: &mut App) {
        let seeded = self
            .initial_hash
            .as_deref()
            .map(UrlHash::parse)
            .unwrap_or_default();

        app.insert_resource(seeded);

        #[cfg(target_arch = "wasm32")]
        {
            wasm::install_listener();
            app.add_systems(Startup, wasm::sync_url_hash);
            app.add_systems(Update, wasm::sync_url_hash);
        }

        // On native, nothing further to wire up — UrlHash stays exactly
        // as seeded from the CLI argument (if any) for the whole run.
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;

    thread_local! {
        static PENDING_HASH: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    pub fn install_listener() {
        let window = web_sys::window().expect("no global `window`");

        let closure = Closure::<dyn FnMut()>::new(move || {
            if let Some(window) = web_sys::window() {
                if let Ok(hash) = window.location().hash() {
                    PENDING_HASH.with(|cell| {
                        *cell.borrow_mut() = Some(hash);
                    });
                }
            }
        });

        window
            .add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref())
            .expect("failed to add hashchange listener");

        closure.forget();
    }

    pub fn sync_url_hash(mut url_hash: ResMut<UrlHash>) {
        let pending = PENDING_HASH.with(|cell| cell.borrow_mut().take());
        let raw = pending.or_else(|| web_sys::window().and_then(|w| w.location().hash().ok()));

        let Some(raw) = raw else { return };

        let parsed = UrlHash::parse(&raw);
        if parsed != *url_hash {
            *url_hash = parsed;
        }
    }
}
