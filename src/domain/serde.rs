use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize_object_id_as_string<S>(id: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Ok(oid) = ObjectId::parse_str(id) {
        serializer.serialize_some(&oid)
    } else {
        serializer.serialize_some(id)
    }
}

pub fn deserialize_object_id_as_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let oid: Option<ObjectId> = Option::deserialize(deserializer)?;
    Ok(oid.map(|o| o.to_hex()))
}

pub fn deserialize_object_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let oid: ObjectId = ObjectId::deserialize(deserializer)?;
    Ok(oid.to_hex())
}

pub fn deserialize_datetime<'de, D>(d: D) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use chrono::TimeZone;
    use chrono::Utc;
    use mongodb::bson::DateTime as BsonDateTime;

    let bson_dt = BsonDateTime::deserialize(d)?;
    let millis = bson_dt.timestamp_millis();
    let date = Utc
        .timestamp_millis_opt(millis)
        .single()
        .ok_or_else(|| serde::de::Error::custom("invalid or ambiguous timestamp"))?;

    Ok(date)
}

pub fn deserialize_option_datetime<'de, D>(
    d: D,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use chrono::TimeZone;
    use mongodb::bson::DateTime as BsonDateTime;

    let opt = Option::<BsonDateTime>::deserialize(d)?;
    dbg!(&opt);
    Ok(opt.and_then(|bson_dt| {
        chrono::Utc
            .timestamp_millis_opt(bson_dt.timestamp_millis())
            .single()
    }))
}
