#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NeuromorphPlane {
    NeuromorphReflex,   // plane: neuromorph.reflex
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NeuromorphDomain {
    ReflexSafetyMicro,      // neuromorph-reflex-micro
    SensoryClarityMicro,    // neuromorph-sense-micro
    AttentionBalanceMicro,  // neuromorph-attention-micro
}

#[derive(Clone, Debug)]
pub struct NeuromorphPlaneTag {
    pub plane: NeuromorphPlane,
    pub domain: NeuromorphDomain,
}
