use std::{error::Error, fmt::Display, ops::Deref};

use sqlx::{
    encode::IsNull,
    postgres::{types::Oid, PgArgumentBuffer, PgTypeInfo},
    Decode, Encode, Postgres, Type,
};
use ulid::Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(pub Ulid);

impl<'r> Decode<'r, Postgres> for Id {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let value = value.as_bytes()?;
        let ulid = Ulid::from_bytes(value.try_into()?);
        Ok(Self(ulid))
    }
}

impl<'r> Encode<'r, Postgres> for Id {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn Error + Send + Sync>> {
        buf.extend_from_slice(self.0.to_bytes().as_ref());
        Ok(IsNull::No)
    }
}

impl Type<Postgres> for Id {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_oid(Oid(17))
    }
}

impl Deref for Id {
    type Target = Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
