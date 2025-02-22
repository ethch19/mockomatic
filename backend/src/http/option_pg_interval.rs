use sqlx::postgres::types::PgInterval;
use serde::ser::{Serializer, SerializeStruct};
use serde::de::{Deserializer, Error, Visitor, SeqAccess, MapAccess};
use std::fmt;

pub fn serialize<S>(interval: &Option<PgInterval>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match interval {
        Some(ref value) => {
            let mut state = serializer.serialize_struct("PgInterval", 3)?;
            state.serialize_field("months", &value.months)?;
            state.serialize_field("days", &value.days)?;
            state.serialize_field("microseconds", &value.microseconds)?;
            state.end()
            },
        None => serializer.serialize_none(),
    }
}

//Make result type an Option
pub fn deserialize<'de, D>(deserializer: D) ->  Result<Option<PgInterval>, D::Error>
where
    D: Deserializer<'de>
{
    #[derive(serde::Deserialize)]
    #[serde(field_identifier, rename_all = "lowercase")]
    enum Field {
        Months,
        Days,
        Microseconds
    }

    struct PgIntervalVisitor;

    impl<'de> Visitor<'de> for PgIntervalVisitor {
        type Value = Option<PgInterval>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("sqlx struct PgInterval")
            }

        fn visit_seq<V>(self, mut seq: V) -> Result<Option<PgInterval>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let months = seq.next_element()?
                .ok_or_else(|| Error::invalid_length(0, &self))?;
            let days = seq.next_element()?
                .ok_or_else(|| Error::invalid_length(1, &self))?;
            let microseconds = seq.next_element()?
                .ok_or_else(|| Error::invalid_length(2, &self))?;
            Ok(Some(PgInterval {
                months,
                days,
                microseconds
            }))
        }
         
        fn visit_map<V>(self, mut map: V) -> Result<Option<PgInterval>, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut months = None;
            let mut days = None;
            let mut microseconds = None;
            while let Some(key) = map.next_key()? {
                match key {
                    Field::Months => {
                        if months.is_some() {
                            return Err(Error::duplicate_field("months"));
                        }
                        months = Some(map.next_value()?);
                    }
                    Field::Days => {
                        if days.is_some() {
                            return Err(Error::duplicate_field("days"));
                        }
                        days = Some(map.next_value()?);
                    }
                    Field::Microseconds => {
                        if microseconds.is_some() {
                            return Err(Error::duplicate_field("microseconds"));
                        }
                        microseconds = Some(map.next_value()?);
                    }
                }
            }
            let months = months.ok_or_else(|| Error::missing_field("months"))?;
            let days = days.ok_or_else(|| Error::missing_field("days"))?;
            let microseconds = microseconds.ok_or_else(|| Error::missing_field("microseconds"))?;
            Ok(Some(PgInterval { months, days, microseconds }))
        }
    }
    
    const FIELDS: &'static[&'static str] = &["months", "days", "microseconds"];
    deserializer.deserialize_struct("PgInterval", FIELDS, PgIntervalVisitor)
}