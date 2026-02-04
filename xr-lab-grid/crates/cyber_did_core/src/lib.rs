#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct StakeholderId {
    pub did: String,
    pub bostrom_primary: String,
    pub googolswarm_root: String,
}
