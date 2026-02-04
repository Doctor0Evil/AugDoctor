use cyber_did_core::StakeholderId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type SessionTokenHashPqc = [u8; 32];

#[derive(Clone, Debug)]
pub struct SessionTokenBinding {
    pub stakeholder: StakeholderId,
    pub token_hash_pqc: SessionTokenHashPqc,
}

pub trait SessionBindingStore: Send + Sync {
    fn bind(&self, binding: SessionTokenBinding);
    fn lookup(&self, token_hash: &SessionTokenHashPqc) -> Option<StakeholderId>;
}

#[derive(Clone, Default)]
pub struct InMemorySessionBindingStore {
    inner: Arc<RwLock<HashMap<SessionTokenHashPqc, StakeholderId>>>,
}

impl SessionBindingStore for InMemorySessionBindingStore {
    fn bind(&self, binding: SessionTokenBinding) {
        let mut map = self.inner.write().expect("session store poisoned");
        map.insert(binding.token_hash_pqc, binding.stakeholder);
    }

    fn lookup(&self, token_hash: &SessionTokenHashPqc) -> Option<StakeholderId> {
        let map = self.inner.read().expect("session store poisoned");
        map.get(token_hash).cloned()
    }
}
