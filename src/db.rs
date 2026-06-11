//! localStorage-backed implementation of the CDK wallet database.
//!
//! The store is (de)serialized from a single localStorage key on every
//! access: reads work on a fresh snapshot, and writes re-read, mutate and
//! write back the blob while holding a cross-tab Web Lock, so several open
//! tabs share one consistent wallet.

use std::collections::HashMap;

use async_trait::async_trait;
use cdk_common::bitcoin::bip32::DerivationPath;
use cdk_common::database::{Error as DbError, WalletDatabase};
use cdk_common::mint_url::MintUrl;
use cdk_common::nuts::{
    CurrencyUnit, Id, KeySet, KeySetInfo, Keys, MintInfo, PaymentMethod, PublicKey,
    SpendingConditions, State,
};
use cdk_common::wallet::{
    MeltQuote, MintQuote, P2PKSigningKey, ProofInfo, Transaction, TransactionDirection,
    TransactionId, WalletSaga,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::storage;

pub const STORAGE_KEY: &str = "cdk_wallet_db";

/// Key the raw store blob is copied to when it can no longer be parsed, so
/// a corrupt (or serde-incompatible after an upgrade) wallet is preserved
/// for recovery instead of being overwritten by the next write.
pub const UNREADABLE_BACKUP_KEY: &str = "cdk_wallet_db_unreadable";

/// Web Lock name guarding read-modify-write cycles on [`STORAGE_KEY`].
#[cfg(target_arch = "wasm32")]
const LOCK_NAME: &str = "cdk_wallet_db";

/// Separator for flattened key-value store keys (unit separator control char).
const KV_SEP: char = '\u{1f}';

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct Store {
    #[serde(default)]
    mints: HashMap<String, Option<MintInfo>>,
    #[serde(default)]
    mint_keysets: HashMap<String, Vec<KeySetInfo>>,
    #[serde(default)]
    keys: HashMap<String, KeySet>,
    #[serde(default)]
    mint_quotes: HashMap<String, MintQuote>,
    #[serde(default)]
    melt_quotes: HashMap<String, MeltQuote>,
    /// Proofs keyed by their Y point (hex).
    #[serde(default)]
    proofs: HashMap<String, ProofInfo>,
    #[serde(default)]
    keyset_counters: HashMap<String, u32>,
    #[serde(default)]
    transactions: HashMap<String, Transaction>,
    #[serde(default)]
    sagas: HashMap<String, WalletSaga>,
    #[serde(default)]
    kv: HashMap<String, Vec<u8>>,
    #[serde(default)]
    p2pk_keys: Vec<P2PKSigningKey>,
}

/// Handle to the wallet database. Carries no state of its own: the store is
/// re-read from localStorage on every access so concurrent tabs stay
/// consistent.
#[derive(Debug, Default)]
pub struct LocalStorageWalletDatabase;

impl LocalStorageWalletDatabase {
    pub fn new() -> Self {
        Self
    }

    fn load() -> Store {
        let Some(raw) = storage::get(STORAGE_KEY) else {
            return Store::default();
        };
        match serde_json::from_str(&raw) {
            Ok(store) => store,
            Err(e) => {
                // Preserve the unreadable blob (keeping the oldest copy) so
                // the next write cannot clobber a possibly recoverable
                // wallet; proofs also remain recoverable via seed restore.
                leptos::logging::error!("wallet store unreadable, backing it up: {e}");
                if storage::get(UNREADABLE_BACKUP_KEY).is_none() {
                    storage::set(UNREADABLE_BACKUP_KEY, &raw);
                }
                Store::default()
            }
        }
    }

    fn save(store: &Store) {
        match serde_json::to_string(store) {
            Ok(json) => {
                if !storage::set(STORAGE_KEY, &json) {
                    leptos::logging::error!(
                        "failed to persist wallet store (localStorage unavailable or quota exceeded)"
                    );
                }
            }
            Err(e) => leptos::logging::error!("failed to serialize wallet store: {e}"),
        }
    }

    /// Run `f` against a fresh snapshot of the store.
    fn read<T>(&self, f: impl FnOnce(&Store) -> T) -> T {
        f(&Self::load())
    }

    /// Run `f` as an atomic read-modify-write cycle: the store is re-read
    /// and written back under a cross-tab Web Lock, so concurrent tabs
    /// cannot interleave with or clobber this update.
    async fn write<T>(&self, f: impl FnOnce(&mut Store) -> T) -> T {
        with_lock(|| {
            let mut store = Self::load();
            let result = f(&mut store);
            Self::save(&store);
            result
        })
        .await
    }

    /// Atomically claim one unspent proof for `unit`, marking it pending
    /// spent in the same locked cycle so two tabs can never take the same
    /// proof.
    pub async fn take_unspent_proof(&self, unit: CurrencyUnit) -> Option<ProofInfo> {
        let unit = Some(unit);
        let state = Some(vec![State::Unspent]);
        self.write(|s| {
            let y = s
                .proofs
                .values()
                .find(|info| info.matches_conditions(&None, &unit, &state, &None))
                .map(|info| info.y.to_string())?;
            let info = s.proofs.get_mut(&y)?;
            info.state = State::PendingSpent;
            Some(info.clone())
        })
        .await
    }
}

/// Run `f` while holding the cross-tab wallet lock. Off wasm (native unit
/// tests) and where the Web Locks API is unavailable, `f` simply runs
/// unlocked (single-tab behavior).
pub async fn with_lock<T>(f: impl FnOnce() -> T) -> T {
    #[cfg(target_arch = "wasm32")]
    let _guard = WebLockGuard::acquire(LOCK_NAME).await;
    f()
}

/// Holder of a Web Lock (cross-tab mutex); the lock is released on drop.
///
/// `navigator.locks.request` keeps the lock until the promise returned by
/// its callback settles, so the guard hangs on to that promise's `resolve`
/// function and calls it when dropped. `acquire` returns `None` when the
/// Locks API is unavailable, in which case the caller proceeds unlocked
/// (single-tab behavior).
#[cfg(target_arch = "wasm32")]
struct WebLockGuard {
    release: Option<js_sys::Function>,
}

#[cfg(target_arch = "wasm32")]
impl WebLockGuard {
    // navigator.locks is reached via Reflect because web-sys gates it
    // behind the unstable-APIs cfg flag.
    async fn acquire(name: &str) -> Option<Self> {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::{JsCast, JsValue};
        use wasm_bindgen_futures::JsFuture;

        let navigator = web_sys::window()?.navigator();
        let locks = js_sys::Reflect::get(navigator.as_ref(), &JsValue::from_str("locks")).ok()?;
        if locks.is_undefined() || locks.is_null() {
            return None;
        }
        let request = js_sys::Reflect::get(&locks, &JsValue::from_str("request"))
            .ok()?
            .dyn_into::<js_sys::Function>()
            .ok()?;

        let mut release = None;
        let held = js_sys::Promise::new(&mut |resolve, _reject| release = Some(resolve));

        let mut signal_granted = None;
        let granted = js_sys::Promise::new(&mut |resolve, _reject| signal_granted = Some(resolve));
        let signal_granted = signal_granted?;

        let callback = Closure::once(move |_lock: JsValue| -> js_sys::Promise {
            let _ = signal_granted.call0(&JsValue::UNDEFINED);
            held
        });
        request
            .call2(&locks, &JsValue::from_str(name), callback.as_ref())
            .ok()?;
        JsFuture::from(granted).await.ok()?;
        Some(Self { release })
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for WebLockGuard {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.call0(&wasm_bindgen::JsValue::UNDEFINED);
        }
    }
}

fn kv_key(primary: &str, secondary: &str, key: &str) -> String {
    format!("{primary}{KV_SEP}{secondary}{KV_SEP}{key}")
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl WalletDatabase<DbError> for LocalStorageWalletDatabase {
    async fn get_mint(&self, mint_url: MintUrl) -> Result<Option<MintInfo>, DbError> {
        Ok(self.read(|s| s.mints.get(&mint_url.to_string()).cloned().flatten()))
    }

    async fn get_mints(&self) -> Result<HashMap<MintUrl, Option<MintInfo>>, DbError> {
        self.read(|s| {
            s.mints
                .iter()
                .map(|(url, info)| Ok((url.parse::<MintUrl>()?, info.clone())))
                .collect::<Result<HashMap<_, _>, cdk_common::mint_url::Error>>()
        })
        .map_err(|e| DbError::Database(Box::new(e)))
    }

    async fn get_mint_keysets(
        &self,
        mint_url: MintUrl,
    ) -> Result<Option<Vec<KeySetInfo>>, DbError> {
        Ok(self.read(|s| s.mint_keysets.get(&mint_url.to_string()).cloned()))
    }

    async fn get_keyset_by_id(&self, keyset_id: &Id) -> Result<Option<KeySetInfo>, DbError> {
        Ok(self.read(|s| {
            s.mint_keysets
                .values()
                .flatten()
                .find(|info| &info.id == keyset_id)
                .cloned()
        }))
    }

    async fn get_mint_quote(&self, quote_id: &str) -> Result<Option<MintQuote>, DbError> {
        Ok(self.read(|s| s.mint_quotes.get(quote_id).cloned()))
    }

    async fn get_mint_quotes(&self) -> Result<Vec<MintQuote>, DbError> {
        Ok(self.read(|s| s.mint_quotes.values().cloned().collect()))
    }

    async fn get_unissued_mint_quotes(&self) -> Result<Vec<MintQuote>, DbError> {
        Ok(self.read(|s| {
            s.mint_quotes
                .values()
                .filter(|quote| {
                    if quote.payment_method == PaymentMethod::BOLT11 {
                        quote.amount_issued == cdk_common::Amount::ZERO
                    } else {
                        true
                    }
                })
                .cloned()
                .collect()
        }))
    }

    async fn get_melt_quote(&self, quote_id: &str) -> Result<Option<MeltQuote>, DbError> {
        Ok(self.read(|s| s.melt_quotes.get(quote_id).cloned()))
    }

    async fn get_melt_quotes(&self) -> Result<Vec<MeltQuote>, DbError> {
        Ok(self.read(|s| s.melt_quotes.values().cloned().collect()))
    }

    async fn get_keys(&self, id: &Id) -> Result<Option<Keys>, DbError> {
        Ok(self.read(|s| s.keys.get(&id.to_string()).map(|ks| ks.keys.clone())))
    }

    async fn get_proofs(
        &self,
        mint_url: Option<MintUrl>,
        unit: Option<CurrencyUnit>,
        state: Option<Vec<State>>,
        spending_conditions: Option<Vec<SpendingConditions>>,
    ) -> Result<Vec<ProofInfo>, DbError> {
        Ok(self.read(|s| {
            s.proofs
                .values()
                .filter(|info| {
                    info.matches_conditions(&mint_url, &unit, &state, &spending_conditions)
                })
                .cloned()
                .collect()
        }))
    }

    async fn get_proofs_by_ys(&self, ys: Vec<PublicKey>) -> Result<Vec<ProofInfo>, DbError> {
        Ok(self.read(|s| {
            ys.iter()
                .filter_map(|y| s.proofs.get(&y.to_string()).cloned())
                .collect()
        }))
    }

    async fn get_balance(
        &self,
        mint_url: Option<MintUrl>,
        unit: Option<CurrencyUnit>,
        state: Option<Vec<State>>,
    ) -> Result<u64, DbError> {
        Ok(self.read(|s| {
            s.proofs
                .values()
                .filter(|info| info.matches_conditions(&mint_url, &unit, &state, &None))
                .map(|info| u64::from(info.proof.amount))
                .sum()
        }))
    }

    async fn get_transaction(
        &self,
        transaction_id: TransactionId,
    ) -> Result<Option<Transaction>, DbError> {
        Ok(self.read(|s| s.transactions.get(&transaction_id.to_string()).cloned()))
    }

    async fn list_transactions(
        &self,
        mint_url: Option<MintUrl>,
        direction: Option<TransactionDirection>,
        unit: Option<CurrencyUnit>,
    ) -> Result<Vec<Transaction>, DbError> {
        Ok(self.read(|s| {
            let mut transactions: Vec<Transaction> = s
                .transactions
                .values()
                .filter(|tx| tx.matches_conditions(&mint_url, &direction, &unit))
                .cloned()
                .collect();
            transactions.sort_by_key(|tx| std::cmp::Reverse(tx.timestamp));
            transactions
        }))
    }

    async fn update_proofs(
        &self,
        added: Vec<ProofInfo>,
        removed_ys: Vec<PublicKey>,
    ) -> Result<(), DbError> {
        self.write(|s| {
            for y in removed_ys {
                s.proofs.remove(&y.to_string());
            }
            for info in added {
                s.proofs.insert(info.y.to_string(), info);
            }
        })
        .await;
        Ok(())
    }

    async fn update_proofs_state(&self, ys: Vec<PublicKey>, state: State) -> Result<(), DbError> {
        self.write(|s| {
            for y in ys {
                if let Some(info) = s.proofs.get_mut(&y.to_string()) {
                    info.state = state;
                }
            }
        })
        .await;
        Ok(())
    }

    async fn add_transaction(&self, transaction: Transaction) -> Result<(), DbError> {
        self.write(|s| {
            s.transactions
                .insert(transaction.id().to_string(), transaction);
        })
        .await;
        Ok(())
    }

    async fn update_mint_url(
        &self,
        old_mint_url: MintUrl,
        new_mint_url: MintUrl,
    ) -> Result<(), DbError> {
        self.write(|s| {
            let old_key = old_mint_url.to_string();
            let new_key = new_mint_url.to_string();
            if let Some(info) = s.mints.remove(&old_key) {
                s.mints.insert(new_key.clone(), info);
            }
            if let Some(keysets) = s.mint_keysets.remove(&old_key) {
                s.mint_keysets.insert(new_key, keysets);
            }
            for proof in s.proofs.values_mut() {
                if proof.mint_url == old_mint_url {
                    proof.mint_url = new_mint_url.clone();
                }
            }
            for quote in s.mint_quotes.values_mut() {
                if quote.mint_url == old_mint_url {
                    quote.mint_url = new_mint_url.clone();
                }
            }
        })
        .await;
        Ok(())
    }

    async fn increment_keyset_counter(&self, keyset_id: &Id, count: u32) -> Result<u32, DbError> {
        Ok(self
            .write(|s| {
                let counter = s.keyset_counters.entry(keyset_id.to_string()).or_insert(0);
                *counter += count;
                *counter
            })
            .await)
    }

    async fn add_mint(
        &self,
        mint_url: MintUrl,
        mint_info: Option<MintInfo>,
    ) -> Result<(), DbError> {
        self.write(|s| {
            s.mints.insert(mint_url.to_string(), mint_info);
        })
        .await;
        Ok(())
    }

    async fn remove_mint(&self, mint_url: MintUrl) -> Result<(), DbError> {
        self.write(|s| {
            let key = mint_url.to_string();
            s.mints.remove(&key);
            s.mint_keysets.remove(&key);
        })
        .await;
        Ok(())
    }

    async fn add_mint_keysets(
        &self,
        mint_url: MintUrl,
        keysets: Vec<KeySetInfo>,
    ) -> Result<(), DbError> {
        self.write(|s| {
            let existing = s.mint_keysets.entry(mint_url.to_string()).or_default();
            for keyset in keysets {
                match existing.iter_mut().find(|k| k.id == keyset.id) {
                    Some(slot) => *slot = keyset,
                    None => existing.push(keyset),
                }
            }
        })
        .await;
        Ok(())
    }

    async fn add_mint_quote(&self, quote: MintQuote) -> Result<(), DbError> {
        // Upsert with optimistic locking, mirroring cdk-sql-common: an
        // update only succeeds if the stored version matches the caller's,
        // and bumps the version.
        self.write(|s| match s.mint_quotes.get_mut(&quote.id) {
            Some(stored) => {
                if stored.version != quote.version {
                    return Err(DbError::ConcurrentUpdate);
                }
                let mut quote = quote;
                quote.version = quote.version.wrapping_add(1);
                *stored = quote;
                Ok(())
            }
            None => {
                s.mint_quotes.insert(quote.id.clone(), quote);
                Ok(())
            }
        })
        .await
    }

    async fn remove_mint_quote(&self, quote_id: &str) -> Result<(), DbError> {
        self.write(|s| {
            s.mint_quotes.remove(quote_id);
        })
        .await;
        Ok(())
    }

    async fn add_melt_quote(&self, quote: MeltQuote) -> Result<(), DbError> {
        // Same optimistic-locking upsert as add_mint_quote.
        self.write(|s| match s.melt_quotes.get_mut(&quote.id) {
            Some(stored) => {
                if stored.version != quote.version {
                    return Err(DbError::ConcurrentUpdate);
                }
                let mut quote = quote;
                quote.version = quote.version.wrapping_add(1);
                *stored = quote;
                Ok(())
            }
            None => {
                s.melt_quotes.insert(quote.id.clone(), quote);
                Ok(())
            }
        })
        .await
    }

    async fn remove_melt_quote(&self, quote_id: &str) -> Result<(), DbError> {
        self.write(|s| {
            s.melt_quotes.remove(quote_id);
        })
        .await;
        Ok(())
    }

    async fn add_keys(&self, keyset: KeySet) -> Result<(), DbError> {
        self.write(|s| {
            s.keys.insert(keyset.id.to_string(), keyset);
        })
        .await;
        Ok(())
    }

    async fn remove_keys(&self, id: &Id) -> Result<(), DbError> {
        self.write(|s| {
            s.keys.remove(&id.to_string());
        })
        .await;
        Ok(())
    }

    async fn remove_transaction(&self, transaction_id: TransactionId) -> Result<(), DbError> {
        self.write(|s| {
            s.transactions.remove(&transaction_id.to_string());
        })
        .await;
        Ok(())
    }

    async fn add_saga(&self, saga: WalletSaga) -> Result<(), DbError> {
        self.write(|s| {
            s.sagas.insert(saga.id.to_string(), saga);
        })
        .await;
        Ok(())
    }

    async fn get_saga(&self, id: &Uuid) -> Result<Option<WalletSaga>, DbError> {
        Ok(self.read(|s| s.sagas.get(&id.to_string()).cloned()))
    }

    async fn update_saga(&self, saga: WalletSaga) -> Result<bool, DbError> {
        // The caller has already incremented saga.version (see
        // WalletSaga::update_state), so the stored row must match the
        // previous version, mirroring cdk-sql-common's
        // `WHERE version = :version - 1` optimistic-locking check.
        Ok(self
            .write(|s| match s.sagas.get_mut(&saga.id.to_string()) {
                Some(stored) if stored.version == saga.version.saturating_sub(1) => {
                    *stored = saga;
                    true
                }
                _ => false,
            })
            .await)
    }

    async fn delete_saga(&self, id: &Uuid) -> Result<(), DbError> {
        self.write(|s| {
            s.sagas.remove(&id.to_string());
        })
        .await;
        Ok(())
    }

    async fn get_incomplete_sagas(&self) -> Result<Vec<WalletSaga>, DbError> {
        // Completed sagas are deleted by the wallet, so everything stored is
        // still in flight.
        Ok(self.read(|s| s.sagas.values().cloned().collect()))
    }

    async fn reserve_proofs(&self, ys: Vec<PublicKey>, operation_id: &Uuid) -> Result<(), DbError> {
        self.write(|s| {
            let keys: Vec<String> = ys.iter().map(|y| y.to_string()).collect();
            for key in &keys {
                match s.proofs.get(key) {
                    Some(info) if info.state == State::Unspent => {}
                    Some(_) => return Err(DbError::ProofNotUnspent),
                    None => return Err(DbError::ProofNotFound),
                }
            }
            for key in &keys {
                if let Some(info) = s.proofs.get_mut(key) {
                    info.state = State::Reserved;
                    info.used_by_operation = Some(*operation_id);
                }
            }
            Ok(())
        })
        .await
    }

    async fn release_proofs(&self, operation_id: &Uuid) -> Result<(), DbError> {
        self.write(|s| {
            for info in s.proofs.values_mut() {
                if info.used_by_operation.as_ref() == Some(operation_id) {
                    info.used_by_operation = None;
                    if info.state == State::Reserved {
                        info.state = State::Unspent;
                    }
                }
            }
        })
        .await;
        Ok(())
    }

    async fn get_reserved_proofs(&self, operation_id: &Uuid) -> Result<Vec<ProofInfo>, DbError> {
        Ok(self.read(|s| {
            s.proofs
                .values()
                .filter(|info| info.used_by_operation.as_ref() == Some(operation_id))
                .cloned()
                .collect()
        }))
    }

    async fn reserve_melt_quote(&self, quote_id: &str, operation_id: &Uuid) -> Result<(), DbError> {
        self.write(|s| match s.melt_quotes.get_mut(quote_id) {
            Some(quote) => match &quote.used_by_operation {
                Some(existing) if existing != &operation_id.to_string() => {
                    Err(DbError::QuoteAlreadyInUse)
                }
                _ => {
                    quote.used_by_operation = Some(operation_id.to_string());
                    Ok(())
                }
            },
            None => Err(DbError::UnknownQuote),
        })
        .await
    }

    async fn release_melt_quote(&self, operation_id: &Uuid) -> Result<(), DbError> {
        self.write(|s| {
            for quote in s.melt_quotes.values_mut() {
                if quote.used_by_operation.as_ref() == Some(&operation_id.to_string()) {
                    quote.used_by_operation = None;
                }
            }
        })
        .await;
        Ok(())
    }

    async fn reserve_mint_quote(&self, quote_id: &str, operation_id: &Uuid) -> Result<(), DbError> {
        self.write(|s| match s.mint_quotes.get_mut(quote_id) {
            Some(quote) => match &quote.used_by_operation {
                Some(existing) if existing != &operation_id.to_string() => {
                    Err(DbError::QuoteAlreadyInUse)
                }
                _ => {
                    quote.used_by_operation = Some(operation_id.to_string());
                    Ok(())
                }
            },
            None => Err(DbError::UnknownQuote),
        })
        .await
    }

    async fn release_mint_quote(&self, operation_id: &Uuid) -> Result<(), DbError> {
        self.write(|s| {
            for quote in s.mint_quotes.values_mut() {
                if quote.used_by_operation.as_ref() == Some(&operation_id.to_string()) {
                    quote.used_by_operation = None;
                }
            }
        })
        .await;
        Ok(())
    }

    async fn kv_read(
        &self,
        primary_namespace: &str,
        secondary_namespace: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, DbError> {
        Ok(self.read(|s| {
            s.kv.get(&kv_key(primary_namespace, secondary_namespace, key))
                .cloned()
        }))
    }

    async fn kv_list(
        &self,
        primary_namespace: &str,
        secondary_namespace: &str,
    ) -> Result<Vec<String>, DbError> {
        Ok(self.read(|s| {
            let prefix = format!("{primary_namespace}{KV_SEP}{secondary_namespace}{KV_SEP}");
            let mut keys: Vec<String> =
                s.kv.keys()
                    .filter_map(|k| k.strip_prefix(&prefix).map(str::to_string))
                    .collect();
            keys.sort();
            keys
        }))
    }

    async fn kv_write(
        &self,
        primary_namespace: &str,
        secondary_namespace: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), DbError> {
        self.write(|s| {
            s.kv.insert(
                kv_key(primary_namespace, secondary_namespace, key),
                value.to_vec(),
            );
        })
        .await;
        Ok(())
    }

    async fn kv_remove(
        &self,
        primary_namespace: &str,
        secondary_namespace: &str,
        key: &str,
    ) -> Result<(), DbError> {
        self.write(|s| {
            s.kv.remove(&kv_key(primary_namespace, secondary_namespace, key));
        })
        .await;
        Ok(())
    }

    async fn add_p2pk_key(
        &self,
        pubkey: &PublicKey,
        derivation_path: DerivationPath,
        derivation_index: u32,
    ) -> Result<(), DbError> {
        self.write(|s| {
            s.p2pk_keys.retain(|key| &key.pubkey != pubkey);
            s.p2pk_keys.push(P2PKSigningKey {
                pubkey: *pubkey,
                derivation_path,
                derivation_index,
                created_time: web_time::SystemTime::now()
                    .duration_since(web_time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            });
        })
        .await;
        Ok(())
    }

    async fn get_p2pk_key(&self, pubkey: &PublicKey) -> Result<Option<P2PKSigningKey>, DbError> {
        Ok(self.read(|s| s.p2pk_keys.iter().find(|k| &k.pubkey == pubkey).cloned()))
    }

    async fn list_p2pk_keys(&self) -> Result<Vec<P2PKSigningKey>, DbError> {
        Ok(self.read(|s| s.p2pk_keys.clone()))
    }

    async fn latest_p2pk(&self) -> Result<Option<P2PKSigningKey>, DbError> {
        Ok(self.read(|s| {
            s.p2pk_keys
                .iter()
                .max_by_key(|k| k.derivation_index)
                .cloned()
        }))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::str::FromStr;

    use cdk_common::nuts::{Id, Proof, PublicKey};
    use cdk_common::secret::Secret;
    use cdk_common::wallet_db_test;
    use cdk_common::Amount;

    use super::*;

    // Each test runs on its own thread (tokio current-thread runtime per
    // #[tokio::test]), and the storage backend is thread-local, so every
    // test sees an isolated empty store.
    async fn provide_db(_test_name: String) -> LocalStorageWalletDatabase {
        LocalStorageWalletDatabase::new()
    }

    // cdk's generic conformance suite for WalletDatabase implementations.
    wallet_db_test!(provide_db);

    fn proof_info(unit: CurrencyUnit) -> ProofInfo {
        let proof = Proof::new(
            Amount::ONE,
            Id::from_str("00916bbf7ef91a36").unwrap(),
            Secret::generate(),
            PublicKey::from_hex(
                "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            )
            .unwrap(),
        );
        ProofInfo::new(
            proof,
            MintUrl::from_str("https://example.com").unwrap(),
            State::Unspent,
            unit,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn take_unspent_proof_claims_atomically() {
        let db = LocalStorageWalletDatabase::new();
        let unit = CurrencyUnit::Custom("xsr".to_string());
        let info = proof_info(unit.clone());
        db.update_proofs(vec![info.clone()], vec![]).await.unwrap();

        let taken = db.take_unspent_proof(unit.clone()).await.unwrap();
        assert_eq!(taken.y, info.y);
        assert_eq!(taken.state, State::PendingSpent);

        // The claimed proof is no longer unspent, so a second take fails.
        assert!(db.take_unspent_proof(unit.clone()).await.is_none());
        let stored = db.get_proofs_by_ys(vec![info.y]).await.unwrap();
        assert_eq!(stored[0].state, State::PendingSpent);
    }

    #[tokio::test]
    async fn unreadable_store_is_backed_up_not_clobbered() {
        let db = LocalStorageWalletDatabase::new();
        storage::set(STORAGE_KEY, "{not json");

        // Reads fall back to an empty store...
        assert_eq!(db.get_balance(None, None, None).await.unwrap(), 0);
        // ...and the raw blob is preserved for recovery.
        assert_eq!(
            storage::get(UNREADABLE_BACKUP_KEY).as_deref(),
            Some("{not json")
        );

        // A later write must not overwrite the backup.
        let unit = CurrencyUnit::Custom("xsr".to_string());
        db.update_proofs(vec![proof_info(unit)], vec![])
            .await
            .unwrap();
        assert_eq!(
            storage::get(UNREADABLE_BACKUP_KEY).as_deref(),
            Some("{not json")
        );
    }
}
