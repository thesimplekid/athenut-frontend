//! Wallet construction and high-level cashu operations built on cdk.

use std::str::FromStr;
use std::sync::{Arc, OnceLock};

use bip39::Mnemonic;
use cdk::amount::SplitTarget;
use cdk::nuts::{CurrencyUnit, PaymentMethod, Proof, PublicKey, State, Token};
use cdk::wallet::{MintQuote, ReceiveOptions, Restored, Wallet};
use cdk::Amount;
use cdk_common::database::WalletDatabase;
use cdk_common::mint_url::MintUrl;
use cdk_common::wallet::ProofInfo;

use crate::db::LocalStorageWalletDatabase;
use crate::storage;

/// Currency unit used by the athenut mint (one token = one search).
pub const UNIT: &str = "xsr";

const SEED_KEY: &str = "seed";
const SEED_BACKUP_KEY: &str = "seed_unreadable";
const MINT_URL_KEY: &str = "mint_url";
const LEGACY_PROOFS_KEY: &str = "proofs";
const LEGACY_PROOFS_BACKUP_KEY: &str = "proofs_unmigrated";
const MIGRATION_PENDING_KEY: &str = "legacy_migration_pending";
const QUOTE_DATES_KEY: &str = "quote_dates";

/// Base URL of the search API. Build-time setting; empty means same origin.
pub fn api_base() -> &'static str {
    option_env!("PUBLIC_API_URL").unwrap_or("")
}

pub fn unit() -> CurrencyUnit {
    CurrencyUnit::Custom(UNIT.to_string())
}

fn db_instance() -> Arc<LocalStorageWalletDatabase> {
    static DB: OnceLock<Arc<LocalStorageWalletDatabase>> = OnceLock::new();
    DB.get_or_init(|| Arc::new(LocalStorageWalletDatabase::new()))
        .clone()
}

pub fn db() -> Arc<dyn WalletDatabase<cdk_common::database::Error> + Send + Sync> {
    db_instance()
}

/// Mint URL: localStorage override, then PUBLIC_API_URL, then the page
/// origin (mint and frontend are served from the same domain by default).
pub fn mint_url() -> String {
    if let Some(url) = storage::get(MINT_URL_KEY) {
        if !url.trim().is_empty() {
            return url;
        }
    }
    let api_base = api_base();
    if !api_base.is_empty() {
        return api_base.to_string();
    }
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default()
}

/// The wallet seed phrase, generating (and persisting) one if absent.
pub fn seed_phrase() -> String {
    if let Some(phrase) = storage::get(SEED_KEY) {
        if Mnemonic::from_str(phrase.trim()).is_ok() {
            return phrase.trim().to_string();
        }
        // Keep the unparseable value around for manual recovery rather
        // than destroying it with the freshly generated replacement.
        storage::set(SEED_BACKUP_KEY, &phrase);
    }
    let mnemonic = Mnemonic::generate(12).expect("mnemonic generation");
    let phrase = mnemonic.to_string();
    storage::set(SEED_KEY, &phrase);
    phrase
}

/// Replace the stored seed phrase after validating it.
pub fn set_seed_phrase(phrase: &str) -> Result<(), String> {
    let mnemonic =
        Mnemonic::from_str(phrase.trim()).map_err(|e| format!("Invalid recovery phrase: {e}"))?;
    storage::set(SEED_KEY, &mnemonic.to_string());
    Ok(())
}

pub fn wallet() -> Result<Wallet, String> {
    let mnemonic = Mnemonic::from_str(&seed_phrase()).map_err(|e| e.to_string())?;
    let seed = mnemonic.to_seed_normalized("");
    Wallet::new(&mint_url(), unit(), db(), seed, None).map_err(|e| e.to_string())
}

/// Current spendable balance (number of searches).
pub async fn balance() -> u64 {
    db().get_balance(None, Some(unit()), Some(vec![State::Unspent]))
        .await
        .unwrap_or(0)
}

/// Import proofs stored by the old cashu-ts frontend (plain JSON array under
/// the "proofs" key) into the cdk database. Offline and instant, so the
/// balance is visible without waiting on the mint; [`finish_legacy_migration`]
/// then reconciles with the mint.
pub async fn import_legacy_proofs() {
    let Some(raw) = storage::get(LEGACY_PROOFS_KEY) else {
        return;
    };
    let Ok(proofs) = serde_json::from_str::<Vec<Proof>>(&raw) else {
        // Keep the raw data around for manual recovery instead of retrying a
        // parse that can never succeed; restore still recovers the funds.
        storage::set(LEGACY_PROOFS_BACKUP_KEY, &raw);
        storage::remove(LEGACY_PROOFS_KEY);
        storage::set(MIGRATION_PENDING_KEY, "1");
        return;
    };
    let Ok(mint) = MintUrl::from_str(&mint_url()) else {
        return;
    };
    let infos: Vec<ProofInfo> = proofs
        .into_iter()
        .filter_map(|proof| ProofInfo::new(proof, mint.clone(), State::Unspent, unit()).ok())
        .collect();
    if db().update_proofs(infos, vec![]).await.is_ok() {
        storage::remove(LEGACY_PROOFS_KEY);
        storage::set(MIGRATION_PENDING_KEY, "1");
    }
}

/// Finish migrating from the old frontend, once per upgrade: restore from the
/// seed (NUT-13), which also moves the keyset counters past the values the
/// old frontend consumed, then mark imported proofs the mint reports as
/// already spent. Kept pending and retried on the next load if the mint is
/// unreachable.
pub async fn finish_legacy_migration() {
    if storage::get(MIGRATION_PENDING_KEY).is_none() {
        return;
    }
    let Ok(wallet) = wallet() else {
        return;
    };
    if wallet.restore().await.is_err() {
        return;
    }
    if mark_spent_proofs(&wallet).await.is_err() {
        return;
    }
    storage::remove(MIGRATION_PENDING_KEY);
}

/// Check all unspent proofs with the mint (NUT-07); cdk drops the ones the
/// mint reports as spent. Restore only adds proofs, so imported legacy
/// proofs that were already spent need this pass to fall out of the balance.
async fn mark_spent_proofs(wallet: &Wallet) -> Result<(), String> {
    let proofs = wallet
        .get_unspent_proofs()
        .await
        .map_err(|e| e.to_string())?;
    if proofs.is_empty() {
        return Ok(());
    }
    wallet
        .check_proofs_spent(proofs)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Settle proofs stuck in `PendingSpent` — a tab closed mid-search, or the
/// backend returned 402 without us knowing whether the proof was redeemed.
/// Asks the mint which ones were actually spent (those are dropped), then
/// swaps the rest into fresh proofs: their tokens already left the browser
/// in a request header, so they must be rotated, not just un-marked. If the
/// swap fails they are left unspent; the mint's double-spend check still
/// protects them, and the next 402 lands back here.
pub async fn reclaim_pending_proofs() {
    let pending_proofs = || async {
        let infos = db()
            .get_proofs(None, Some(unit()), Some(vec![State::PendingSpent]), None)
            .await
            .unwrap_or_default();
        infos
            .into_iter()
            .filter(|info| info.used_by_operation.is_none())
            .collect::<Vec<_>>()
    };
    if pending_proofs().await.is_empty() {
        return;
    }
    let Ok(wallet) = wallet() else {
        return;
    };
    // Drops the proofs the mint reports as already spent.
    if wallet.check_all_pending_proofs().await.is_err() {
        return;
    }
    let survivors = pending_proofs().await;
    if survivors.is_empty() {
        return;
    }
    let ys: Vec<PublicKey> = survivors.iter().map(|info| info.y).collect();
    if db().update_proofs_state(ys, State::Unspent).await.is_err() {
        return;
    }
    let _ = wallet
        .swap(
            None,
            SplitTarget::Value(Amount::ONE),
            survivors.into_iter().map(|info| info.proof).collect(),
            None,
            false,
            false,
        )
        .await;
}

/// Take one unspent proof out of the wallet for a search request, returning
/// its Y point (for settling later) and the encoded cashu token. The proof
/// is claimed atomically under a cross-tab lock so two tabs can never send
/// the same proof.
pub async fn take_search_proof() -> Result<(PublicKey, String), String> {
    let info = db_instance()
        .take_unspent_proof(unit())
        .await
        .ok_or("No proofs available")?;
    let y = info.y;
    let token = Token::new(info.mint_url, vec![info.proof], None, unit());
    Ok((y, token.to_string()))
}

/// Settle a search proof after the request finished.
pub async fn settle_search_proof(y: PublicKey, spent: bool) {
    let state = if spent { State::Spent } else { State::Unspent };
    let _ = db().update_proofs_state(vec![y], state).await;
}

/// Create a bolt11 mint quote for the given number of searches.
pub async fn create_mint_quote(searches: u64) -> Result<MintQuote, String> {
    let wallet = wallet()?;
    let quote = wallet
        .mint_quote(
            PaymentMethod::BOLT11,
            Some(Amount::from(searches)),
            None,
            None,
        )
        .await
        .map_err(|e| e.to_string())?;
    record_quote_date(&quote.id).await;
    Ok(quote)
}

/// Check the state of a mint quote with the mint.
pub async fn check_mint_quote(quote_id: &str) -> Result<MintQuote, String> {
    let wallet = wallet()?;
    wallet
        .check_mint_quote_status(quote_id)
        .await
        .map_err(|e| e.to_string())
}

/// Mint proofs for a paid quote, splitting into denomination-1 proofs so each
/// proof pays for exactly one search.
pub async fn mint_proofs(quote_id: &str) -> Result<u64, String> {
    let wallet = wallet()?;
    let proofs = wallet
        .mint(quote_id, SplitTarget::Value(Amount::ONE), None)
        .await
        .map_err(|e| e.to_string())?;
    Ok(proofs.len() as u64)
}

/// All locally known mint quotes (the topup page invoice history).
pub async fn pending_quotes() -> Vec<MintQuote> {
    let mut quotes = db().get_mint_quotes().await.unwrap_or_default();
    let dates = prune_quote_dates(&quotes).await;
    quotes.sort_by(|a, b| {
        let a_date = dates.get(&a.id).copied().unwrap_or(0.0);
        let b_date = dates.get(&b.id).copied().unwrap_or(0.0);
        b_date
            .partial_cmp(&a_date)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    quotes
}

/// Creation timestamps (ms since epoch) for quotes, kept outside the cdk
/// database because the cdk quote type does not record creation time.
pub fn quote_dates() -> std::collections::HashMap<String, f64> {
    storage::get(QUOTE_DATES_KEY)
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_quote_dates(dates: &std::collections::HashMap<String, f64>) {
    if let Ok(json) = serde_json::to_string(dates) {
        storage::set(QUOTE_DATES_KEY, &json);
    }
}

async fn record_quote_date(quote_id: &str) {
    crate::db::with_lock(|| {
        let mut dates = quote_dates();
        dates.insert(quote_id.to_string(), js_sys::Date::now());
        save_quote_dates(&dates);
    })
    .await;
}

/// Drop dates for quotes that no longer exist so the map cannot grow
/// unboundedly, returning the (pruned) dates.
async fn prune_quote_dates(quotes: &[MintQuote]) -> std::collections::HashMap<String, f64> {
    crate::db::with_lock(|| {
        let mut dates = quote_dates();
        let before = dates.len();
        dates.retain(|id, _| quotes.iter().any(|quote| &quote.id == id));
        if dates.len() != before {
            save_quote_dates(&dates);
        }
        dates
    })
    .await
}

/// Redeem an encoded cashu token into the wallet.
pub async fn receive_token(token: &str) -> Result<Amount, String> {
    let wallet = wallet()?;
    wallet
        .receive(
            token.trim(),
            ReceiveOptions {
                amount_split_target: SplitTarget::Value(Amount::ONE),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

/// Restore wallet proofs from the seed phrase via the mint (NUT-13).
pub async fn restore() -> Result<Restored, String> {
    let wallet = wallet()?;
    wallet.restore().await.map_err(|e| e.to_string())
}

/// Encode the current mint's unspent proofs as a single cashu token for
/// export. A token can only carry one mint, so proofs left over from a
/// different (previously configured) mint are not included.
pub async fn export_token() -> Result<String, String> {
    let mint = MintUrl::from_str(&mint_url()).map_err(|e| e.to_string())?;
    let proofs = db()
        .get_proofs(
            Some(mint.clone()),
            Some(unit()),
            Some(vec![State::Unspent]),
            None,
        )
        .await
        .map_err(|e| e.to_string())?;
    if proofs.is_empty() {
        return Ok(String::new());
    }
    let token = Token::new(
        mint,
        proofs.into_iter().map(|info| info.proof).collect(),
        None,
        unit(),
    );
    Ok(token.to_string())
}

/// Amount in sats of a bolt11 invoice.
pub fn invoice_amount_sats(invoice: &str) -> Option<u64> {
    let invoice = cdk_common::Bolt11Invoice::from_str(invoice).ok()?;
    invoice
        .amount_milli_satoshis()
        .map(|msats| msats.div_ceil(1000))
}
