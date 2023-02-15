/// Serialzies and deserialzies durations and time marks into nanoseconds
pub mod ts_nanos {
    use serde::{*, de::Visitor};
    use std::time::{Duration, SystemTime};

    pub trait NanosSer {
        fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer;
    }

    pub trait NanosDeser<'de>: Sized {
        fn deserialize<D>(deser: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>;
    }

    impl NanosSer for SystemTime {
        #[inline]
        fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            NanosSer::serialize(
                &self
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(serde::ser::Error::custom)?,
                ser,
            )
        }
    }

    impl<'de> NanosDeser<'de> for SystemTime {
        #[inline]
        fn deserialize<D>(deser: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            SystemTime::UNIX_EPOCH.checked_add(<Duration as NanosDeser::<'de>>::deserialize(deser)?)
                .ok_or_else(|| serde::de::Error::custom("overflow when adding duration to system time"))
        }
    }

    impl NanosSer for Duration {
        #[inline]
        fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            cfg_if::cfg_if! {
                if #[cfg(no_integer128)] {
                    return match u64::try_from(self.as_nanos()) {
                        Ok(x) => <u64 as Serialize>::serialize(&dur.as_nanos(), ser),
                        Err(e) => Err(serde::ser::Error::custom(e))
                    }
                } else {
                    return <u128 as Serialize>::serialize(&self.as_nanos(), ser)
                }
            }
        }
    }

    impl<'de> NanosDeser<'de> for Duration {
        #[inline]
        fn deserialize<D>(deser: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {

            struct LocalVisitor;
            impl<'de> Visitor<'de> for LocalVisitor {
                type Value = Duration;

                #[inline]
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "a nanosecond timestamp")
                }

                #[inline]
                fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: de::Error, {
                    return Ok(Duration::from_nanos(v))
                }

                serde_if_integer128! {
                    #[inline]
                    fn visit_u128<E> (self, v: u128) -> Result<Self::Value,E> where E:de::Error, {
                        const NANOS_PER_SECOND: u128 = 1_000_000_000;
                        match u64::try_from(v / NANOS_PER_SECOND) {
                            Ok(secs) => {
                                let nanos = (v % NANOS_PER_SECOND) as u32;
                                return Ok(Duration::new(secs, nanos))
                            },
                            Err(e) => return Err(serde::de::Error::custom(e))
                        }
                    }
                }
            }

            deser.deserialize_any(LocalVisitor)
        }
    }

    #[inline]
    pub fn serialize<T: NanosSer, S>(this: &T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        T::serialize(this, ser)
    }

    #[inline]
    pub fn deserialize<'de, T: NanosDeser<'de>, D>(deser: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deser)
    }
}
