#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Status codes returned by `verify`. Kept as `u32` so off-chain verifiers can
// consume the result without parsing Soroban enums.
// ---------------------------------------------------------------------------

/// No credential has been issued for the queried address.
pub const STATUS_NONE: u32 = 0;
/// Credential exists, was issued by the registered authority, and is within
/// its validity window.
pub const STATUS_VALID: u32 = 1;
/// Credential's `valid_until` is in the past. Computed at call time.
pub const STATUS_EXPIRED: u32 = 2;
/// Credential was permanently revoked by the issuing authority.
pub const STATUS_REVOKED: u32 = 3;
/// Credential was temporarily suspended by the issuing authority and may be
/// reinstated later.
pub const STATUS_SUSPENDED: u32 = 4;

// ---------------------------------------------------------------------------
// Storage types
// ---------------------------------------------------------------------------

/// On-chain record of a citizen's digital citizenship credential.
#[contracttype]
pub struct Credential {
    /// Jurisdiction code (e.g. `"US"`, `"EU"`, `"VN"`).
    pub jurisdiction: Symbol,
    /// Ledger timestamp at which the credential was issued.
    pub issued_at: u64,
    /// Ledger timestamp after which the credential is treated as expired.
    pub valid_until: u64,
    /// Persisted status: one of `STATUS_VALID`, `STATUS_REVOKED`,
    /// `STATUS_SUSPENDED`. `STATUS_NONE` and `STATUS_EXPIRED` are derived.
    pub status: u32,
    /// Free-text reason supplied by the authority on revoke/suspend.
    pub reason: Symbol,
}

/// Storage keys used by the contract.
#[contracttype]
pub enum DataKey {
    /// The government authority permitted to mutate credentials. Stored in
    /// instance storage because it is set once at `initialize`.
    Authority,
    /// A citizen's credential, keyed by their on-chain address. Stored in
    /// persistent storage so that state survives contract invocation.
    Credential(Address),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// `CitizenPaper` — a digital citizenship credential registry.
///
/// A single registered government authority issues, revokes, suspends, and
/// reinstates credentials on behalf of citizens. Off-chain verifiers (KYC
/// services, exchanges, etc.) call `verify` to look up the current status
/// of a citizen without needing to talk to the authority directly.
#[contract]
pub struct CitizenPaper;

#[contractimpl]
impl CitizenPaper {
    /// Initialize the contract by registering the government `authority`
    /// that will be permitted to mutate credentials. Must be called exactly
    /// once before any of the other contract functions. Requires
    /// authorization from `authority` so that the address registered is
    /// provably the one calling.
    pub fn initialize(env: Env, authority: Address) {
        authority.require_auth();
        if env.storage().instance().has(&DataKey::Authority) {
            panic!("contract already initialized");
        }
        env.storage().instance().set(&DataKey::Authority, &authority);
    }

    /// Issue a new digital citizenship credential to `citizen` for the
    /// given `jurisdiction`, valid through `valid_until` (unix timestamp in
    /// seconds). Panics if a credential already exists for the citizen or
    /// if `valid_until` is not strictly in the future. Requires
    /// authorization from the registered authority.
    pub fn issue(
        env: Env,
        authority: Address,
        citizen: Address,
        jurisdiction: Symbol,
        valid_until: u64,
    ) {
        authority.require_auth();
        Self::require_authority(&env, &authority);

        let key = DataKey::Credential(citizen);
        if env.storage().persistent().has(&key) {
            panic!("credential already exists for citizen");
        }

        let now = env.ledger().timestamp();
        if valid_until <= now {
            panic!("valid_until must be in the future");
        }

        let credential = Credential {
            jurisdiction,
            issued_at: now,
            valid_until,
            status: STATUS_VALID,
            reason: Symbol::new(&env, ""),
        };
        env.storage().persistent().set(&key, &credential);
    }

    /// Permanently revoke the credential of `citizen`, recording `reason`
    /// for off-chain auditability. A revoked credential cannot be
    /// reinstated. Requires authorization from the registered authority.
    pub fn revoke(env: Env, authority: Address, citizen: Address, reason: Symbol) {
        authority.require_auth();
        Self::require_authority(&env, &authority);
        let mut credential = Self::load_credential(&env, &citizen);
        credential.status = STATUS_REVOKED;
        credential.reason = reason;
        env.storage()
            .persistent()
            .set(&DataKey::Credential(citizen), &credential);
    }

    /// Temporarily suspend the credential of `citizen`, recording `reason`.
    /// A suspended credential reports `STATUS_SUSPENDED` from `verify`
    /// until it is reinstated with a fresh `valid_until`. Requires
    /// authorization from the registered authority.
    pub fn suspend(env: Env, authority: Address, citizen: Address, reason: Symbol) {
        authority.require_auth();
        Self::require_authority(&env, &authority);
        let mut credential = Self::load_credential(&env, &citizen);
        credential.status = STATUS_SUSPENDED;
        credential.reason = reason;
        env.storage()
            .persistent()
            .set(&DataKey::Credential(citizen), &credential);
    }

    /// Reinstate a previously suspended credential for `citizen` and extend
    /// its validity to `new_valid_until`. Panics if `new_valid_until` is
    /// not strictly in the future. Requires authorization from the
    /// registered authority.
    pub fn reinstate(
        env: Env,
        authority: Address,
        citizen: Address,
        new_valid_until: u64,
    ) {
        authority.require_auth();
        Self::require_authority(&env, &authority);
        let mut credential = Self::load_credential(&env, &citizen);
        let now = env.ledger().timestamp();
        if new_valid_until <= now {
            panic!("new_valid_until must be in the future");
        }
        credential.status = STATUS_VALID;
        credential.valid_until = new_valid_until;
        credential.reason = Symbol::new(&env, "");
        env.storage()
            .persistent()
            .set(&DataKey::Credential(citizen), &credential);
    }

    /// Verify the current status of `citizen`'s credential. Returns one of
    /// the `STATUS_*` codes defined at the top of this file. Expiry is
    /// computed at call time using the ledger timestamp, so a credential
    /// whose `valid_until` has elapsed returns `STATUS_EXPIRED` even
    /// though `status` on disk is still `STATUS_VALID`.
    pub fn verify(env: Env, citizen: Address) -> u32 {
        let key = DataKey::Credential(citizen);
        let credential: Credential = match env.storage().persistent().get(&key) {
            Some(c) => c,
            None => return STATUS_NONE,
        };
        match credential.status {
            STATUS_REVOKED => STATUS_REVOKED,
            STATUS_SUSPENDED => STATUS_SUSPENDED,
            _ => {
                if env.ledger().timestamp() > credential.valid_until {
                    STATUS_EXPIRED
                } else {
                    STATUS_VALID
                }
            }
        }
    }

    /// Return the jurisdiction code recorded on `citizen`'s credential.
    /// Panics if no credential has been issued for the address. Useful
    /// for verifiers that need to know not just *whether* a citizen is
    /// valid but also *under which jurisdiction* they were issued.
    pub fn get_jurisdiction(env: Env, citizen: Address) -> Symbol {
        Self::load_credential(&env, &citizen).jurisdiction
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

impl CitizenPaper {
    /// Assert that `authority` matches the address registered at
    /// `initialize`. Panics otherwise.
    fn require_authority(env: &Env, authority: &Address) {
        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Authority)
            .expect("contract not initialized");
        if stored != *authority {
            panic!("caller is not the registered authority");
        }
    }

    /// Load a citizen's credential, panicking if it does not exist.
    fn load_credential(env: &Env, citizen: &Address) -> Credential {
        env.storage()
            .persistent()
            .get(&DataKey::Credential(citizen.clone()))
            .expect("no credential for citizen")
    }
}
