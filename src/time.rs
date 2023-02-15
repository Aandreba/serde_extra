macro_rules! impl_u128 {
    ($(
        mod $mod:ident: $name:literal as $fn:ident | $from:ident | $delta:literal {
            pub trait $ser:ident;
            pub trait $de:ident;
        }
    )+) => {
        $(
            #[doc = concat!("Serialzies and deserialzies durations and time marks into ", $name)]
            pub mod $mod {
                use serde::{*, de::Visitor};
                use std::time::{Duration, SystemTime};

                pub trait $ser {
                    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
                    where
                        S: Serializer;
                }

                pub trait $de<'de>: Sized {
                    fn deserialize<D>(deser: D) -> Result<Self, D::Error>
                    where
                        D: Deserializer<'de>;
                }

                impl $ser for SystemTime {
                    #[inline]
                    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
                    where
                        S: Serializer,
                    {
                        $ser::serialize(
                            &self
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map_err(serde::ser::Error::custom)?,
                            ser,
                        )
                    }
                }

                impl<'de> $de<'de> for SystemTime {
                    #[inline]
                    fn deserialize<D>(deser: D) -> Result<Self, D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        SystemTime::UNIX_EPOCH.checked_add(<Duration as $de::<'de>>::deserialize(deser)?)
                            .ok_or_else(|| serde::de::Error::custom("overflow when adding duration to system time"))
                    }
                }

                impl $ser for Duration {
                    #[inline]
                    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
                    where
                        S: Serializer,
                    {
                        cfg_if::cfg_if! {
                            if #[cfg(all(not(feature = "use_128"), no_integer128))] {
                                return match u64::try_from(self.$fn()) {
                                    Ok(x) => <u64 as Serialize>::serialize(&x, ser),
                                    Err(e) => Err(serde::ser::Error::custom(e))
                                }
                            } else {
                                return <u128 as Serialize>::serialize(&self.$fn(), ser)
                            }
                        }
                    }
                }

                impl<'de> $de<'de> for Duration {
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
                                write!(formatter, concat!("a ", $name, " timestamp"))
                            }

                            #[inline]
                            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: de::Error, {
                                return Ok(Duration::$from(v))
                            }

                            #[inline]
                            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> where E: de::Error, {
                                const DELTA: f32 = 1f32 / ($delta as f32);
                                return Duration::try_from_secs_f32(DELTA * v).map_err(serde::de::Error::custom)
                            }

                            #[inline]
                            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> where E: de::Error, {
                                const DELTA: f64 = 1f64 / ($delta as f64);
                                return Duration::try_from_secs_f64(DELTA * v).map_err(serde::de::Error::custom)
                            }

                            #[cfg(feature = "use_128")]
                            serde_if_integer128! {
                                #[inline]
                                fn visit_u128<E> (self, v: u128) -> Result<Self::Value,E> where E:de::Error, {
                                    match u64::try_from(v / $delta) {
                                        Ok(secs) => {
                                            let nanos = (v % $delta) as u32;
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
                pub fn serialize<T: $ser, S>(this: &T, ser: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    T::serialize(this, ser)
                }

                #[inline]
                pub fn deserialize<'de, T: $de<'de>, D>(deser: D) -> Result<T, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    T::deserialize(deser)
                }
            }
        )+
    };
}

impl_u128! {
    mod ts_nanos: "nanoseconds" as as_nanos | from_nanos | 1_000_000_000 {
        pub trait SerializeNanos;
        pub trait DeserializeNanos;
    }

    mod ts_micros: "microseconds" as as_micros | from_micros | 1_000_000 {
        pub trait SerializeMicros;
        pub trait DeserializeMicros;
    }

    mod ts_millis: "milliseconds" as as_millis | from_millis | 1_000 {
        pub trait SerializeMillis;
        pub trait DeserializeMillis;
    }
}

#[doc = "Serialzies and deserialzies durations and time marks into seconds"]
pub mod ts_secs {
    use serde::{*, de::Visitor};
    use std::time::{Duration, SystemTime};

    pub trait SerializeSecs {
        fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer;
    }

    pub trait DeserializeSecs<'de>: Sized {
        fn deserialize<D>(deser: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>;
    }

    impl SerializeSecs for SystemTime {
        #[inline]
        fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            SerializeSecs::serialize(
                &self
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(serde::ser::Error::custom)?,
                ser,
            )
        }
    }

    impl<'de> DeserializeSecs<'de> for SystemTime {
        #[inline]
        fn deserialize<D>(deser: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            SystemTime::UNIX_EPOCH.checked_add(<Duration as DeserializeSecs::<'de>>::deserialize(deser)?)
                .ok_or_else(|| serde::de::Error::custom("overflow when adding duration to system time"))
        }
    }

    impl SerializeSecs for Duration {
        #[inline]
        fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            <u64 as Serialize>::serialize(&self.as_secs(), ser)
        }
    }

    impl<'de> DeserializeSecs<'de> for Duration {
        #[inline]
        fn deserialize<D>(deser: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct LocalVisitor;
            impl<'de> Visitor<'de> for LocalVisitor {
                type Value = Duration;

                #[inline]
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "a second timestamp")
                }

                #[inline]
                fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: de::Error, {
                    return Ok(Duration::from_secs(v))
                }

                #[inline]
                fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> where E: de::Error, {
                    return Duration::try_from_secs_f32(v).map_err(serde::de::Error::custom)
                }

                #[inline]
                fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> where E: de::Error, {
                    return Duration::try_from_secs_f64(v).map_err(serde::de::Error::custom)
                }
            }

            <u64 as Deserialize<'de>>::deserialize(deser).map(Duration::from_secs)
        }
    }

    #[inline]
    pub fn serialize<T: SerializeSecs, S>(this: &T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        T::serialize(this, ser)
    }

    #[inline]
    pub fn deserialize<'de, T: DeserializeSecs<'de>, D>(deser: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deser)
    }
}