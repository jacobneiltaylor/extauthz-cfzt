pub trait AudienceProvider: Sync + Send {
    fn get_audiences(&self) -> Vec<String>;
}

pub struct StaticAudienceProvider {
    audiences: Vec<String>,
}

impl StaticAudienceProvider {
    pub fn new(audiences: Vec<String>) -> Self {
        StaticAudienceProvider { audiences }
    }

    pub fn new_single_aud(audience: &str) -> Self {
        Self::new(vec![audience.to_string()])
    }
}

impl AudienceProvider for StaticAudienceProvider {
    fn get_audiences(&self) -> Vec<String> {
        self.audiences.clone()
    }
}
