use sqlx::postgres::{PgHasArrayType, PgTypeInfo};

#[derive(Debug, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "endpoint_kind", rename_all = "lowercase")]
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

impl PgHasArrayType for EndpointKind {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_endpoint_kind")
    }
}
