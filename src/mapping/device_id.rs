#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId {
    pub vendor: u16,
    pub product: u16,
    pub version: u16,
}

#[cfg(feature = "serde")]
impl serde::Serialize for DeviceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!(
            "{:04x}:{:04x}:{:04x}",
            self.vendor, self.product, self.version
        );

        return serializer.serialize_str(&s);
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DeviceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let mut parts = s.split(':');

        let vendor = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing vendor"))?;

        let product = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing product"))?;

        let version = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing version"))?;

        if parts.next().is_some() {
            return Err(serde::de::Error::custom("too many fields"));
        }

        return Ok(DeviceId {
            vendor: u16::from_str_radix(vendor, 16).map_err(serde::de::Error::custom)?,
            product: u16::from_str_radix(product, 16).map_err(serde::de::Error::custom)?,
            version: u16::from_str_radix(version, 16).map_err(serde::de::Error::custom)?,
        });
    }
}
