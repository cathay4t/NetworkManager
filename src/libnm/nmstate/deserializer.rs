// SPDX-License-Identifier: Apache-2.0

// This file is based on the work of nmstate project(https://nmstate.io/) which
// is under license of Apache 2.0, authors of original file are:
//  * Gris Ge <fge@redhat.com>
//  * Fernando Fernandez Mancera <ffmancera@riseup.net>
//  * Wen Liang <liangwen12year@gmail.com>

use std::{marker::PhantomData, str::FromStr};

use serde::{Deserializer, de, de::Visitor};

pub(crate) fn u8_or_string<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    option_u8_or_string(deserializer).and_then(|i| {
        if let Some(i) = i {
            Ok(i)
        } else {
            Err(de::Error::custom("Required filed undefined"))
        }
    })
}

pub(crate) fn u16_or_string<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    option_u16_or_string(deserializer).and_then(|i| {
        if let Some(i) = i {
            Ok(i)
        } else {
            Err(de::Error::custom("Required filed undefined"))
        }
    })
}

pub(crate) fn u32_or_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    option_u32_or_string(deserializer).and_then(|i| {
        if let Some(i) = i {
            Ok(i)
        } else {
            Err(de::Error::custom("Required filed undefined"))
        }
    })
}

pub(crate) fn bool_or_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    option_bool_or_string(deserializer).and_then(|i| {
        if let Some(i) = i {
            Ok(i)
        } else {
            Err(de::Error::custom("Required filed undefined"))
        }
    })
}

pub(crate) fn option_bool_or_string<'de, D>(
    deserializer: D,
) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    struct IntegerOrString(PhantomData<fn() -> Option<bool>>);

    impl Visitor<'_> for IntegerOrString {
        type Value = Option<bool>;

        fn expecting(
            &self,
            formatter: &mut std::fmt::Formatter,
        ) -> std::fmt::Result {
            formatter.write_str(
                "Need to be boolean: 1|0|true|false|yes|no|on|off|y|n",
            )
        }

        fn visit_bool<E>(self, value: bool) -> Result<Option<bool>, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<bool>, E>
        where
            E: de::Error,
        {
            match value.to_lowercase().as_str() {
                "1" | "true" | "yes" | "on" | "y" => Ok(Some(true)),
                "0" | "false" | "no" | "off" | "n" => Ok(Some(false)),
                _ => Err(de::Error::custom(
                    "Need to be boolean: 1|0|true|false|yes|no|on|off|y|n",
                )),
            }
        }

        fn visit_u64<E>(self, value: u64) -> Result<Option<bool>, E>
        where
            E: de::Error,
        {
            match value {
                1 => Ok(Some(true)),
                0 => Ok(Some(false)),
                _ => Err(de::Error::custom(
                    "Need to be boolean: 1|0|true|false|yes|no|on|off|y|n",
                )),
            }
        }
    }

    deserializer.deserialize_any(IntegerOrString(PhantomData))
}

pub(crate) fn option_u8_or_string<'de, D>(
    deserializer: D,
) -> Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    option_u64_or_string(deserializer).and_then(|i| {
        if let Some(i) = i {
            match u8::try_from(i) {
                Ok(i) => Ok(Some(i)),
                Err(e) => Err(de::Error::custom(e)),
            }
        } else {
            Ok(None)
        }
    })
}

pub(crate) fn option_u16_or_string<'de, D>(
    deserializer: D,
) -> Result<Option<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    option_u64_or_string(deserializer).and_then(|i| {
        if let Some(i) = i {
            match u16::try_from(i) {
                Ok(i) => Ok(Some(i)),
                Err(e) => Err(de::Error::custom(e)),
            }
        } else {
            Ok(None)
        }
    })
}

pub(crate) fn option_u32_or_string<'de, D>(
    deserializer: D,
) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    option_u64_or_string(deserializer).and_then(|i| {
        if let Some(i) = i {
            match u32::try_from(i) {
                Ok(i) => Ok(Some(i)),
                Err(e) => Err(de::Error::custom(e)),
            }
        } else {
            Ok(None)
        }
    })
}

// This function is inspired by https://serde.rs/string-or-struct.html
pub(crate) fn option_u64_or_string<'de, D>(
    deserializer: D,
) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct IntegerOrString(PhantomData<fn() -> Option<u64>>);

    impl Visitor<'_> for IntegerOrString {
        type Value = Option<u64>;

        fn expecting(
            &self,
            formatter: &mut std::fmt::Formatter,
        ) -> std::fmt::Result {
            formatter.write_str("unsigned integer or string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<u64>, E>
        where
            E: de::Error,
        {
            if let Some(prefix_len) = value.strip_prefix("0x") {
                u64::from_str_radix(prefix_len, 16)
                    .map_err(de::Error::custom)
                    .map(Some)
            } else {
                FromStr::from_str(value)
                    .map_err(de::Error::custom)
                    .map(Some)
            }
        }

        fn visit_u64<E>(self, value: u64) -> Result<Option<u64>, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }
    }

    deserializer.deserialize_any(IntegerOrString(PhantomData))
}

pub(crate) fn option_i32_or_string<'de, D>(
    deserializer: D,
) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    option_i64_or_string(deserializer).and_then(|i| {
        if let Some(i) = i {
            match i32::try_from(i) {
                Ok(i) => Ok(Some(i)),
                Err(e) => Err(de::Error::custom(e)),
            }
        } else {
            Ok(None)
        }
    })
}

pub(crate) fn option_i64_or_string<'de, D>(
    deserializer: D,
) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct IntegerOrString(PhantomData<fn() -> Option<i64>>);

    impl Visitor<'_> for IntegerOrString {
        type Value = Option<i64>;

        fn expecting(
            &self,
            formatter: &mut std::fmt::Formatter,
        ) -> std::fmt::Result {
            formatter.write_str("signed integer or string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<i64>, E>
        where
            E: de::Error,
        {
            FromStr::from_str(value)
                .map_err(de::Error::custom)
                .map(Some)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Option<i64>, E>
        where
            E: de::Error,
        {
            i64::try_from(value).map_err(de::Error::custom).map(Some)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Option<i64>, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }
    }

    deserializer.deserialize_any(IntegerOrString(PhantomData))
}

pub(crate) fn option_number_as_string<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct NumberOrString(PhantomData<fn() -> Option<String>>);

    impl Visitor<'_> for NumberOrString {
        type Value = Option<String>;

        fn expecting(
            &self,
            formatter: &mut std::fmt::Formatter,
        ) -> std::fmt::Result {
            formatter.write_str("signed integer or string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<String>, E>
        where
            E: de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Option<String>, E>
        where
            E: de::Error,
        {
            Ok(Some(format!("{}", value)))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Option<String>, E>
        where
            E: de::Error,
        {
            Ok(Some(format!("{}", value)))
        }

        fn visit_f64<E>(self, value: f64) -> Result<Option<String>, E>
        where
            E: de::Error,
        {
            Ok(Some(format!("{}", value)))
        }
    }

    deserializer.deserialize_any(NumberOrString(PhantomData))
}
