use sqlx::PgExecutor;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "endpoint_kind")]
#[sqlx(rename_all = "lowercase")]
pub enum EndpointKind {
    Peer,
    Seed,
    Rpc,
    Rest,
    Grpc,
}

impl EndpointKind {
    pub(crate) fn as_field(&self) -> &str {
        match self {
            EndpointKind::Peer => "persistent_peers",
            EndpointKind::Seed => "seeds",
            EndpointKind::Rpc => "rpc",
            EndpointKind::Rest => "rest",
            EndpointKind::Grpc => "grpc",
        }
    }
}
